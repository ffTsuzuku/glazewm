use std::str::FromStr;

use anyhow::bail;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(
  Clone,
  Copy,
  Debug,
  Default,
  Deserialize,
  Eq,
  PartialEq,
  Serialize,
  ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum ContainerLayout {
  #[default]
  Tiles,
  #[serde(alias = "stacked")]
  Accordion,
}

impl ContainerLayout {
  /// Toggles between `Tiles` and `Accordion` layouts.
  ///
  /// # Example
  /// ```
  /// # use wm_common::ContainerLayout;
  /// let layout = ContainerLayout::Tiles.toggle();
  /// assert_eq!(layout, ContainerLayout::Accordion);
  /// ```
  #[must_use]
  pub fn toggle(&self) -> Self {
    match self {
      Self::Tiles => Self::Accordion,
      Self::Accordion => Self::Tiles,
    }
  }
}

impl FromStr for ContainerLayout {
  type Err = anyhow::Error;

  /// Parses a string into a container layout.
  ///
  /// # Example
  /// ```
  /// # use wm_common::ContainerLayout;
  /// # use std::str::FromStr;
  /// let layout = ContainerLayout::from_str("accordion");
  /// assert_eq!(layout.unwrap(), ContainerLayout::Accordion);
  ///
  /// let layout = ContainerLayout::from_str("stacked");
  /// assert_eq!(layout.unwrap(), ContainerLayout::Accordion);
  ///
  /// let layout = ContainerLayout::from_str("tiles");
  /// assert_eq!(layout.unwrap(), ContainerLayout::Tiles);
  /// ```
  fn from_str(unparsed: &str) -> anyhow::Result<Self> {
    match unparsed {
      "tiles" => Ok(Self::Tiles),
      "accordion" | "stacked" => Ok(Self::Accordion),
      _ => bail!("Not a valid container layout: {}", unparsed),
    }
  }
}
