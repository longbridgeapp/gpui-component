pub trait Disableable {
    fn disabled(self, disabled: bool) -> Self;
}
