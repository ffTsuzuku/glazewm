use ambassador::delegatable_trait;
use wm_platform::Rect;

#[delegatable_trait]
pub trait PositionGetters {
  fn to_rect(&self) -> anyhow::Result<Rect>;
}

/// Computes `(l_padding, r_padding)` for a child in an accordion layout.
///
/// Follows `AeroSpace`'s accordion layout:
/// - Inactive windows peek out by `padding` pixels at the container edges.
/// - The focused (MRU) window occupies the main area.
/// - Windows retain nearly full container dimensions to avoid triggering
///   minimum window size limits.
#[must_use]
pub fn calculate_accordion_padding(
  count: usize,
  own_index: usize,
  mru_index: usize,
  padding: i32,
) -> (i32, i32) {
  if count <= 1 || padding <= 0 {
    return (0, 0);
  }

  if own_index == 0 {
    (0, padding)
  } else if own_index == count - 1 {
    (padding, 0)
  } else if own_index + 1 == mru_index {
    (0, 2 * padding)
  } else if own_index == mru_index + 1 {
    (2 * padding, 0)
  } else {
    (padding, padding)
  }
}

/// Implements the `PositionGetters` trait for tiling containers that can
/// be resized. This is used by `SplitContainer` and `TilingWindow`.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with an `id` and a `parent` field.
#[macro_export]
macro_rules! impl_position_getters_as_resizable {
  ($struct_name:ident) => {
    impl PositionGetters for $struct_name {
      #[allow(clippy::too_many_lines)]
      fn to_rect(&self) -> anyhow::Result<Rect> {
        let parent = self
          .parent()
          .and_then(|parent| parent.as_direction_container().ok())
          .context("Parent does not have a tiling direction.")?;

        let parent_rect = parent.to_rect()?;

        if parent.layout() == wm_common::ContainerLayout::Accordion {
          let tiling_children =
            parent.tiling_children().collect::<Vec<_>>();
          let count = tiling_children.len();

          let own_index = tiling_children
            .iter()
            .position(|c| c.id() == self.id())
            .context("Child container not found in parent.")?;

          let mru_index = parent
            .child_focus_order()
            .find_map(|child| {
              tiling_children.iter().position(|c| c.id() == child.id())
            })
            .unwrap_or(0);

          let monitor = self.monitor().context("No monitor.")?;
          let gaps_config = self.gaps_config();
          let scale_factor = if gaps_config.scale_with_dpi {
            monitor.native_properties().scale_factor
          } else {
            1.
          };

          let (l_padding, r_padding) = match parent.tiling_direction() {
            TilingDirection::Horizontal => {
              let padding = gaps_config
                .accordion_padding
                .to_px(parent_rect.width(), Some(scale_factor));
              $crate::traits::calculate_accordion_padding(
                count, own_index, mru_index, padding,
              )
            }
            TilingDirection::Vertical => {
              let padding = gaps_config
                .accordion_padding
                .to_px(parent_rect.height(), Some(scale_factor));
              $crate::traits::calculate_accordion_padding(
                count, own_index, mru_index, padding,
              )
            }
          };

          let (x, y, width, height) = match parent.tiling_direction() {
            TilingDirection::Horizontal => {
              let x = parent_rect.x() + l_padding;
              let y = parent_rect.y();
              let width =
                (parent_rect.width() - l_padding - r_padding).max(1);
              let height = parent_rect.height();
              (x, y, width, height)
            }
            TilingDirection::Vertical => {
              let x = parent_rect.x();
              let y = parent_rect.y() + l_padding;
              let width = parent_rect.width();
              let height =
                (parent_rect.height() - l_padding - r_padding).max(1);
              (x, y, width, height)
            }
          };

          return Ok(Rect::from_xy(x, y, width, height));
        }

        let (horizontal_gap, vertical_gap) = self.inner_gaps()?;
        let inner_gap = match parent.tiling_direction() {
          TilingDirection::Vertical => vertical_gap,
          TilingDirection::Horizontal => horizontal_gap,
        };

        #[allow(
          clippy::cast_precision_loss,
          clippy::cast_possible_truncation,
          clippy::cast_possible_wrap
        )]
        let (width, height) = match parent.tiling_direction() {
          TilingDirection::Vertical => {
            let available_height = parent_rect.height()
              - inner_gap * self.tiling_siblings().count() as i32;

            let height =
              (self.tiling_size() * available_height as f32) as i32;

            (parent_rect.width(), height)
          }
          TilingDirection::Horizontal => {
            let available_width = parent_rect.width()
              - inner_gap * self.tiling_siblings().count() as i32;

            let width =
              (available_width as f32 * self.tiling_size()).round() as i32;

            (width, parent_rect.height())
          }
        };

        let (x, y) = {
          let mut prev_siblings = self
            .prev_siblings()
            .filter_map(|sibling| sibling.as_tiling_container().ok());

          match prev_siblings.next() {
            None => (parent_rect.x(), parent_rect.y()),
            Some(sibling) => {
              let sibling_rect = sibling.to_rect()?;

              match parent.tiling_direction() {
                TilingDirection::Vertical => (
                  parent_rect.x(),
                  sibling_rect.y() + sibling_rect.height() + inner_gap,
                ),
                TilingDirection::Horizontal => (
                  sibling_rect.x() + sibling_rect.width() + inner_gap,
                  parent_rect.y(),
                ),
              }
            }
          }
        };

        Ok(Rect::from_xy(x, y, width, height))
      }
    }
  };
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_accordion_padding_single_child() {
    assert_eq!(calculate_accordion_padding(1, 0, 0, 30), (0, 0));
  }

  #[test]
  fn test_accordion_padding_two_children() {
    // Child 0 focused
    assert_eq!(calculate_accordion_padding(2, 0, 0, 30), (0, 30));
    assert_eq!(calculate_accordion_padding(2, 1, 0, 30), (30, 0));

    // Child 1 focused
    assert_eq!(calculate_accordion_padding(2, 0, 1, 30), (0, 30));
    assert_eq!(calculate_accordion_padding(2, 1, 1, 30), (30, 0));
  }

  #[test]
  fn test_accordion_padding_three_children() {
    // Child 1 focused (in middle)
    assert_eq!(calculate_accordion_padding(3, 0, 1, 30), (0, 30));
    assert_eq!(calculate_accordion_padding(3, 1, 1, 30), (30, 30));
    assert_eq!(calculate_accordion_padding(3, 2, 1, 30), (30, 0));

    // Child 0 focused (first)
    assert_eq!(calculate_accordion_padding(3, 0, 0, 30), (0, 30));
    assert_eq!(calculate_accordion_padding(3, 1, 0, 30), (60, 0));
    assert_eq!(calculate_accordion_padding(3, 2, 0, 30), (30, 0));

    // Child 2 focused (last)
    assert_eq!(calculate_accordion_padding(3, 0, 2, 30), (0, 30));
    assert_eq!(calculate_accordion_padding(3, 1, 2, 30), (0, 60));
    assert_eq!(calculate_accordion_padding(3, 2, 2, 30), (30, 0));
  }

  #[test]
  fn test_accordion_zero_padding() {
    assert_eq!(calculate_accordion_padding(3, 0, 1, 0), (0, 0));
    assert_eq!(calculate_accordion_padding(3, 1, 1, 0), (0, 0));
  }
}
