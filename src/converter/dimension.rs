//! Dimensions for ASCII and image sizes.
//!
//! The [Dimension] struct is used to represent the dimension of the ASCII input
//! and the image. Dimensions will be scaled up or down in order to generate ASCII
//! and images that are of a reasonable size for humans to view.
//!
//! Robert Peterson and Kelsey Werner 2023

/// [Dimension] is a struct that holds image dimension information.
#[derive(Debug, PartialEq)]
pub struct Dimension {
    /// The width of the item.
    pub width: u32,
    /// The height of the item.
    pub height: u32,
}

impl Dimension {
    /// Create a new [Dimension] of size (0,0).
    pub fn new() -> Self {
        Dimension {
            width: 0,
            height: 0,
        }
    }

    /// Create a new [Dimension] from  an existing tuple of ([u32], [u32]).
    pub fn from(t: (u32, u32)) -> Self {
        Dimension {
            width: t.0,
            height: t.1,
        }
    }

    /// Scale down the dimensions so neither dimension is larger than `max`.
    ///
    /// Scaling the dimensions will preserve the dimension ratio.
    pub fn scale_down(&mut self, max: u32) {
        if self.width <= max && self.height <= max {
            return;
        }
        self.scale(max);
    }

    /// Scale up the dimensions so the largest dimension is at least `min`.
    ///
    /// Scaling the dimensions will preserve the dimension ratio.
    pub fn scale_up(&mut self, min: u32) {
        if self.width >= min || self.height >= min {
            return;
        }
        self.scale(min);
    }

    /// Scale the dimensions by a given `factor`.
    ///
    /// Scaling the dimensions will preserve the dimension ratio.
    /// The math algorithm to do this was found here:
    ///     https://tutors.com/lesson/what-is-a-scale-factor
    fn scale(&mut self, factor: u32) {
        let w: u32;
        let h: u32;

        if self.width > self.height {
            w = factor;
            let ratio = self.height as f32 / self.width as f32;
            h = (factor as f32 * ratio) as u32;
        } else {
            h = factor;
            let ratio = self.width as f32 / self.height as f32;
            w = (factor as f32 * ratio) as u32;
        }

        self.width = w;
        self.height = h;
    }
}

impl Default for Dimension {
    /// The default construction of a [Dimension].
    fn default() -> Self {
        Self::new()
    }
}

// Test that a [Dimension] can be created from a tuple.
#[test]
fn test_from() {
    let d = Dimension::from((150, 500));
    assert_eq!(150, d.width);
    assert_eq!(500, d.height);
}

// Test that the [Dimension] can be scaled down.
#[test]
fn test_scale_down() {
    // changes dimensions
    let mut d1 = Dimension::from((150, 500));
    d1.scale_down(300);
    assert_eq!(90, d1.width);
    assert_eq!(300, d1.height);

    // does not change dimensions
    let mut d2 = Dimension::from((200, 190));
    d2.scale_down(200);
    assert_eq!(200, d2.width);
    assert_eq!(190, d2.height);
}

// Test that the [Dimension] can be scaled up.
#[test]
fn test_scape_down() {
    // changes dimensions
    let mut d1 = Dimension::from((300, 90));
    d1.scale_up(500);
    assert_eq!(500, d1.width);
    assert_eq!(150, d1.height);

    // does not change dimensions
    let mut d2 = Dimension::from((300, 90));
    d2.scale_up(90);
    assert_eq!(300, d2.width);
    assert_eq!(90, d2.height);
}
