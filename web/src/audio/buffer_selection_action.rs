use super::buffer_selection::BufferSelection;

pub trait BufferSelectionAction {
    const BUFFER_SELECTION_START: f32;
    const BUFFER_SELECTION_END: f32;
    const BUFFER_SELECTION_MIN_LEN: f32;

    fn sanitize_selection(start_or_end: f32) -> f32 {
        start_or_end
            .max(Self::BUFFER_SELECTION_START)
            .min(Self::BUFFER_SELECTION_END)
    }

    fn set_mouse_start(&mut self, start: f32) -> &mut Self;

    fn set_mouse_end(&mut self, end: f32) -> &mut Self;

    fn set_mouse_down(&mut self, mouse_down: bool) -> &mut Self;

    fn get_mouse_down(&self) -> bool;

    fn get_mouse_start(&self) -> f32;

    fn get_mouse_end(&self) -> f32;

    /// Copies the existing buffer selection struct out
    fn get_buffer_selection(&self) -> BufferSelection;

    /// Returns the mouse start / mouse end poisition in the correct order
    /// (i.e. from least to greatest / from left to right)
    ///
    /// This does not guarantee that the start and end are not the SAME number.
    fn get_buffer_start_and_end(&self) -> (f32, f32);

    /// Returns the mouse start position (this number is guarnateed to be <= the end position)
    fn get_buffer_start(&self) -> f32 {
        self.get_buffer_start_and_end().0
    }

    /// Returns the mouse end poisitino (this number is guarnateed to be >= the start position)
    fn get_buffer_end(&self) -> f32 {
        self.get_buffer_start_and_end().1
    }
}
