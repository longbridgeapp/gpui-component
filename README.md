# GPUI App Example

This is an example of build app by using GPUI.

<https://github.com/huacnlee/gpui-app/assets/5518/ad103f02-697a-40ed-a876-8b13e4242a72>

<https://github.com/huacnlee/gpui-app/assets/5518/5316f9f0-58c8-4b99-bd79-eafffb38c3fc>

<https://github.com/huacnlee/gpui-app/assets/5518/0273e031-4426-4ab5-a41c-f7cbbb0e55bc>

## Demo

[gpui-app-windows.zip](https://github.com/user-attachments/files/16039599/main-app.zip) - Updated at 2024/06/29

## TODO

- [x] Theming
- [ ] TextField
  - [x] Ctrl+a, e to move cursor to start/end
  - [x] Copy, Cut, Paste by keyboard
  - [ ] ContextMenu to let user copy, cut, paste
  - [ ] Selection by mouse, drag to select text
  - [x] Cursor blinking
- [x] Button
  - [x] Button with Icon
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
- [x] Dropdown
  - [ ] Combobox
  - [x] Picker
    - [x] Picker List
    - [x] Use keyword to select next, prev, enter to select, esc to cancel.
- [x] Tabs
  - [x] Tab
  - [x] TabBar
- [x] Notification
- [x] Tooltip
- [x] Popover
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
- picker
- popover (based on popup_menu)

> I think we can discuss them with Zed team to change the license to MIT or Apache License for share to community.

Other code are under Apache License.
