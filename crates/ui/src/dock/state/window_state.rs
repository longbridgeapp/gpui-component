use gpui::{Bounds, Pixels, Size};

/// The state of a [`Surface::Window`](crate::Surface::Window).
///
/// Doubles as a handle for the surface, allowing the user to set its size and position.
#[derive(Clone, Debug)]
pub struct WindowState {
    /// The [`Size`] that this window was last taking up.
    screen_size: Option<Size<Pixels>>,

    /// Was this window dragged in the last frame?
    dragged: bool,

    next_bounds: Option<Bounds<Pixels>>,

    /// true the first frame this window is drawn.
    /// handles opening collapsing header, etc.
    new: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            screen_size: None,
            dragged: false,
            next_bounds: None,
            new: true,
        }
    }
}

impl WindowState {
    /// Create a default window state.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub fn set_bounds(&mut self, bounds: Bounds<Pixels>) -> &mut Self {
        self.next_bounds = Some(bounds);
        self
    }

    /// Get the [`Rect`] which this window occupies.
    /// If this window hasn't been shown before, this will be [`Rect::NOTHING`].
    pub fn size(&self) -> Size<Pixels> {
        // The reason why we're unwrapping an Option with a default value instead of
        // just storing Rect::NOTHING for the None variant is that deserializing Rect::NOTHING
        // with serde_json causes a panic, because f32::INFINITY serializes into null in JSON.
        self.screen_size.unwrap_or(Size::default())
    }

    /// Returns if this window is currently being dragged or not.
    pub fn dragged(&self) -> bool {
        self.dragged
    }

    pub(crate) fn next_bounds(&mut self) -> Option<Bounds<Pixels>> {
        self.next_bounds.take()
    }

    // //the 'static in this case means that the `open` field is always `None`
    // pub(crate) fn create_window(&mut self, id: Id, bounds: Rect) -> (egui::Window<'static>, bool) {
    //     let new = self.new;
    //     let mut window_constructor = egui::Window::new("")
    //         .id(id)
    //         .constrain_to(bounds)
    //         .title_bar(false);

    //     if let Some(position) = self.next_position() {
    //         window_constructor = window_constructor.current_pos(position);
    //     }
    //     if let Some(size) = self.next_size() {
    //         window_constructor = window_constructor.fixed_size(size);
    //     }
    //     self.new = false;
    //     (window_constructor, new)
    // }
}
