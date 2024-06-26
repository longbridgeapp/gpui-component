# GPUI App Example

This is an example of build app by using GPUI.

https://github.com/huacnlee/gpui-app/assets/5518/ad103f02-697a-40ed-a876-8b13e4242a72

## TODO

- [x] Theming
- [ ] TextField
  - [x] Ctrl+a, e to move cursor to start/end
  - [x] Copy, Cut, Paste by keyboard
  - [ ] ContextMenu to let user copy, cut, paste
  - [ ] Selection by mouse, drag to select text
  - [ ] Cursor blink
- [x] Button
  - [ ] IconButton
  - [ ] ToggleButton
- [x] Label
- [x] Icon
- [x] Checkbox
  - [x] With label
- [x] Switch
  - [x] With Label (Left, Right side)
  - [ ] Toggle Animation
- [ ] Radio & RadioGroup
- [ ] Dropdown
  - [ ] Combobox
- [x] Tabs
  - [x] Tab
  - [x] TabBar
- [ ] Dockpanel & Splitter
- [ ] List
- [ ] Table

## How to build

```bash
cargo run
```

## License

There have a part of UI components from [Zed](https://github.com/zed-industries/zed/tree/main/crates/ui), that are under GPL v3.0 license.

- title_bar

> I think we can discuss them with Zed team to change the license to MIT or Apache License for share to community.

Other code are under Apache License.
