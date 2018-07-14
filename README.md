# bidrag
A rust library for managing bindable game input. Currently only KB/M, but will have support for joysticks in the future.

Example (Using GLFW):
``` rust
let mut glfw = glfw3::init(glfw3::FAIL_ON_ERRORS).expect("Failed to init GLFW3");
let (mut window, events) = {
    glfw.window_hint(glfw3::WindowHint::Visible(true));

    let windowMode = glfw3::WindowMode::Windowed;
    // return
    glfw.create_window(1600, 900, APP_NAME, windowMode).expect("Failed to create a window!")
};
let mut cursorMode = glfw3::CursorMode::Disabled; // For mouse input

window.set_cursor_pos_polling(true);
window.set_cursor_mode(cursorMode);
window.set_key_polling(true);
window.make_current();

let mut inputSub = bidrag::InputSubsystem::new((2.0, 2.0));
// Axes are referred to by indices, rather than using a string lookup every time.
let yawIndex = inputSub.add_binding("Yaw".to_string(), Binding::MAxis
    (MouseAxis::X));
let quitIndex = inputSub.add_binding("Quit".to_string(), Binding::KBKey
    (glfw3::Key::Escape as i32));

let mut running = true;
while running {
    glfw.poll_events();
    for (_, ev) in glfw3::flush_messages(&events) {
        match ev {
            glfw3::WindowEvent::Key(key, _, action, _) => {
                // For now you manually update the binds, since I wanted this API-agnostic
                inputSub.update_kb_bind(key as i32, action == glfw3::Action::Press);
            },
            _ => {}
        }
    }

    inputSub.update_mouseaxes_bind(window.get_cursor_pos());

    running = running && !window.should_close();
}
```
TODO:
- Joysticks
- (Possibly) multiple KB/M.
- - At the very least, the library can support it and leave it up to the client
