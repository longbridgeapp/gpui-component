mod button_story;
mod input_story;
mod story;

pub use story::Stories;
use story::StoryContainer;

pub fn story_case(name: &'static str, description: &'static str) -> StoryContainer {
    StoryContainer::new(name, description)
}
