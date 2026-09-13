use anyhow::Context;
use wm_common::ContainerLayout;

use crate::{
  models::{Container, DirectionContainer, TilingWindow},
  traits::{CommonGetters, TilingDirectionGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Toggles the container layout between `Tiles` and `Accordion`.
pub fn toggle_layout(
  container: Container,
  state: &mut WmState,
  _config: &UserConfig,
) -> anyhow::Result<()> {
  let direction_container = match container {
    Container::TilingWindow(ref tiling_window) => {
      toggle_window_layout(tiling_window)?
    }
    Container::Workspace(workspace) => {
      workspace.set_layout(workspace.layout().toggle());
      workspace.into()
    }
    Container::Split(split) => {
      split.set_layout(split.layout().toggle());
      split.into()
    }
    _ => return Ok(()),
  };

  state
    .pending_sync
    .queue_container_to_redraw(direction_container);

  Ok(())
}

fn toggle_window_layout(
  tiling_window: &TilingWindow,
) -> anyhow::Result<DirectionContainer> {
  let parent = tiling_window
    .direction_container()
    .context("No direction container.")?;

  parent.set_layout(parent.layout().toggle());

  Ok(parent)
}

/// Sets the container layout to a given `ContainerLayout`.
pub fn set_layout(
  container: Container,
  state: &mut WmState,
  config: &UserConfig,
  layout: ContainerLayout,
) -> anyhow::Result<()> {
  let direction_container = match container {
    Container::TilingWindow(ref tiling_window) => tiling_window
      .direction_container()
      .context("No direction container.")?,
    Container::Workspace(ref workspace) => workspace.clone().into(),
    Container::Split(ref split) => split.clone().into(),
    _ => return Ok(()),
  };

  if direction_container.layout() == layout {
    Ok(())
  } else {
    toggle_layout(container, state, config)
  }
}

#[cfg(test)]
mod tests {
  use wm_common::ContainerLayout;

  use super::*;
  use crate::models::{SplitContainer, Workspace};

  #[test]
  fn toggles_workspace_layout() {
    let mut state = WmState::mock();
    let config = UserConfig::mock();
    let workspace = Workspace::mock().call();

    assert_eq!(workspace.layout(), ContainerLayout::Tiles);

    toggle_layout(workspace.clone().into(), &mut state, &config).unwrap();
    assert_eq!(workspace.layout(), ContainerLayout::Accordion);
    assert_eq!(state.pending_sync.containers_to_redraw().len(), 1);

    toggle_layout(workspace.clone().into(), &mut state, &config).unwrap();
    assert_eq!(workspace.layout(), ContainerLayout::Tiles);
  }

  #[test]
  fn toggles_split_layout() {
    let mut state = WmState::mock();
    let config = UserConfig::mock();
    let split = SplitContainer::mock().call();

    assert_eq!(split.layout(), ContainerLayout::Tiles);

    toggle_layout(split.clone().into(), &mut state, &config).unwrap();
    assert_eq!(split.layout(), ContainerLayout::Accordion);
  }

  #[test]
  fn set_layout_updates_or_ignores() {
    let mut state = WmState::mock();
    let config = UserConfig::mock();
    let workspace = Workspace::mock().call();

    assert_eq!(workspace.layout(), ContainerLayout::Tiles);

    // Setting same layout is no-op
    set_layout(
      workspace.clone().into(),
      &mut state,
      &config,
      ContainerLayout::Tiles,
    )
    .unwrap();
    assert_eq!(workspace.layout(), ContainerLayout::Tiles);
    assert_eq!(state.pending_sync.containers_to_redraw().len(), 0);

    // Setting different layout updates it
    set_layout(
      workspace.clone().into(),
      &mut state,
      &config,
      ContainerLayout::Accordion,
    )
    .unwrap();
    assert_eq!(workspace.layout(), ContainerLayout::Accordion);
    assert_eq!(state.pending_sync.containers_to_redraw().len(), 1);
  }
}
