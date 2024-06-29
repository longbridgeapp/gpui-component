use gpui::{ClickEvent, Focusable, InteractiveElement, WindowContext};

pub trait InterativeElementExt: InteractiveElement {
    /// Set the listener for a double click event.
    fn on_double_click(
        mut self,
        listener: impl Fn(&ClickEvent, &mut WindowContext) + 'static,
    ) -> Self
    where
        Self: Sized,
    {
        self.interactivity().on_click(move |event, context| {
            if event.up.click_count == 2 {
                listener(event, context);
            }
        });
        self
    }
}

impl<E: InteractiveElement> InterativeElementExt for Focusable<E> {}

// impl<E> InterativeElementExt for Stateful<E>
// where
//     E: Element,
//     Self: InteractiveElement,
// {
// }
