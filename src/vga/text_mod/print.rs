use super::out;

fn finalize_write(
    screen_index: usize,
    top_line_before: usize,
    changed_cell: Option<(usize, usize)>,
    force_full_redraw: bool,
) {
    out::sync_screen_state(screen_index);

    let top_line_after = out::visible_top_line(screen_index);
    if force_full_redraw || top_line_after != top_line_before {
        out::render_screen(screen_index);
        return;
    }

    if let Some((line, column)) = changed_cell {
        out::render_cell_if_visible(screen_index, line, column);
    }

    out::sync_cursor(screen_index);
}

fn newline_with_scroll(screen_index: usize) -> bool {
    let mut cursor = out::cursor_of_screen(screen_index);
    cursor.x = 0;
    let mut force_full_redraw = false;

    let next_line = usize::from(cursor.y) + 1;
    if next_line >= out::SCROLLBACK_LINES {
        out::shift_buffer_up(screen_index);
        cursor.y = (out::SCROLLBACK_LINES - 1) as u16;
        force_full_redraw = true;
    } else {
        cursor.y = next_line as u16;
    }

    let cursor_line = usize::from(cursor.y);
    let used_lines = out::used_lines_of_screen(screen_index);
    if cursor_line + 1 > used_lines {
        out::set_used_lines(screen_index, cursor_line + 1);
        out::clear_buffer_line(screen_index, cursor_line);
    }

    out::set_cursor(screen_index, usize::from(cursor.x), usize::from(cursor.y));

    force_full_redraw
}

fn backspace(screen_index: usize) -> Option<(usize, usize)> {
    let mut cursor = out::cursor_of_screen(screen_index);
    if cursor.x > 0 {
        cursor.x -= 1;
    } else if cursor.y > 0 {
        cursor.y -= 1;
        cursor.x = (out::VGA_WIDTH - 1) as u16;
    } else {
        return None;
    }

    let line = usize::from(cursor.y);
    let column = usize::from(cursor.x);
    out::set_cursor(screen_index, usize::from(cursor.x), usize::from(cursor.y));
    out::write_blank_cell(screen_index, line, column);

    Some((line, column))
}

fn write_byte(byte: u8) {
    let screen_index = out::current_screen_index();
    let top_line_before = out::visible_top_line(screen_index);

    match byte {
        b'\n' => {
            let force_full_redraw = newline_with_scroll(screen_index);
            finalize_write(screen_index, top_line_before, None, force_full_redraw);
        }
        b'\r' => {
            out::set_cursor_x(screen_index, 0);
            finalize_write(screen_index, top_line_before, None, false);
        }
        0x08 => {
            if let Some(changed_cell) = backspace(screen_index) {
                finalize_write(screen_index, top_line_before, Some(changed_cell), false);
            }
        }
        b'\t' => {
            for _ in 0..4 {
                write_byte(b' ');
            }
        }
        byte => {
            let cursor = out::cursor_of_screen(screen_index);
            let line = usize::from(cursor.y);
            let column = usize::from(cursor.x);
            let vga_char = (byte as u16) | ((unsafe { out::CURRENT_COLOR.0 } as u16) << 8);
            out::write_cell(screen_index, line, column, vga_char);

            let next_x = column + 1;
            if next_x >= out::VGA_WIDTH {
                let force_full_redraw = newline_with_scroll(screen_index);
                finalize_write(
                    screen_index,
                    top_line_before,
                    Some((line, column)),
                    force_full_redraw,
                );
            } else {
                out::set_cursor(screen_index, next_x, line);
                finalize_write(screen_index, top_line_before, Some((line, column)), false);
            }
        }
    }
}

pub fn write_str(text: &str) {
    for &byte in text.as_bytes() {
        write_byte(byte);
    }
}

pub fn write_char(c: char) {
    let byte = if (c as u32) <= 0xFF { c as u8 } else { b'?' };
    write_byte(byte);
}

pub fn newline() {
    write_byte(b'\n');
}
