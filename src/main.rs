use std::io;

mod cli;

use cli::FileData;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout as TuiLayout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

fn main() -> io::Result<()> {
    let mut app = App::load();
    let mut terminal = ratatui::init();
    let result = app.run(&mut terminal);
    ratatui::restore();
    result
}

struct Theme {
    text: Color,
    background: Color,
    highlight: Color,
    shadow: Color,
}

const THEME_KEY_BASE: Theme = Theme {
    text: Color::Rgb(16, 24, 48),
    background: Color::Rgb(48, 72, 144),
    highlight: Color::Rgb(64, 96, 192),
    shadow: Color::Rgb(32, 48, 96),
};

const THEME_KEY_HINT: Theme = Theme {
    text: Color::Rgb(16, 48, 16),
    background: Color::Rgb(48, 144, 48),
    highlight: Color::Rgb(64, 192, 64),
    shadow: Color::Rgb(32, 96, 32),
};

struct Key {
    theme: &'static Theme,
    text: Line<'static>,
}

impl Widget for &Key {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // Set the style for the button
        buf.set_style(
            area,
            Style::new().bg(self.theme.background).fg(self.theme.text),
        );
        // Render the top line
        if area.height > 2 {
            buf.set_string(
                area.x,
                area.y,
                " ".repeat(area.width as usize),
                Style::new()
                    .fg(self.theme.highlight)
                    .bg(self.theme.background),
            )
        }
        // Render the bottom line
        if area.height > 1 {
            buf.set_string(
                area.x,
                area.y + area.height - 1,
                "▁".repeat(area.width as usize),
                Style::new().fg(self.theme.shadow).bg(self.theme.background),
            );
        }
        // Render the label
        let margin_x = area.width.saturating_sub(self.text.width() as u16) / 2;
        let margin_y = area.height.saturating_sub(1) / 2;
        buf.set_line(area.x + margin_x, area.y + margin_y, &self.text, area.width);
    }
}

pub struct Keyboard {
    layout: &'static Layout,
    keys: Vec<Vec<Key>>,
    draw: bool,
    sym: Key,
    cur: Key,
    shift: Key,
}

impl Default for Keyboard {
    fn default() -> Self {
        Keyboard::from_layout(&LAYOUT_QWERTY)
    }
}

impl Keyboard {
    fn toggle_draw(&mut self) {
        self.draw = !self.draw;
    }
    fn next_layout(&mut self) {
        if std::ptr::eq(self.layout, &LAYOUT_QWERTY) {
            self.set_dvorak();
        } else if std::ptr::eq(self.layout, &LAYOUT_DVORAK) {
            self.set_3l();
        } else if std::ptr::eq(self.layout, &LAYOUT_3L) {
            self.set_qwerty();
        }
    }
    fn from_layout(layout: &'static Layout) -> Self {
        let mut keys = vec![];
        for row in layout.base {
            let mut row_keys = vec![];
            for key in *row {
                let text = if *key == '\0' {
                    Line::from("").centered()
                } else {
                    Line::from(key.to_string().bold().white()).centered()
                };
                row_keys.push(Key {
                    theme: &THEME_KEY_BASE,
                    text,
                })
            }
            keys.push(row_keys)
        }
        Self {
            keys,
            layout,
            draw: true,
            cur: Key {
                theme: &THEME_KEY_BASE,
                text: Line::from("cur".to_string().bold().white()).centered(),
            },
            sym: Key {
                theme: &THEME_KEY_BASE,
                text: Line::from("sym".to_string().bold().white()).centered(),
            },
            shift: Key {
                theme: &THEME_KEY_BASE,
                text: Line::from("shift".to_string().bold().white()).centered(),
            },
        }
    }
    fn set_qwerty(&mut self) {
        *self = Self::from_layout(&LAYOUT_QWERTY)
    }
    fn set_dvorak(&mut self) {
        *self = Self::from_layout(&LAYOUT_DVORAK)
    }
    fn set_3l(&mut self) {
        *self = Self::from_layout(&LAYOUT_3L)
    }

    fn update(&mut self, c: char) {
        for key in self.keys.iter_mut().flatten() {
            key.theme = &THEME_KEY_BASE;
        }
        for modifier in [&mut self.sym, &mut self.cur, &mut self.shift] {
            modifier.theme = &THEME_KEY_BASE;
        }
        let Some(location) = self.layout.location(c) else {
            return;
        };
        if let Some(row) = self.keys.get_mut(location.row as usize) {
            if let Some(key) = row.get_mut(location.col as usize) {
                key.theme = &THEME_KEY_HINT
            }
        }
        match location.modifier {
            Some(Modifier::Sym) => &mut self.sym,
            Some(Modifier::Cur) => &mut self.cur,
            Some(Modifier::Shift) => &mut self.shift,
            None => return,
        }
        .theme = &THEME_KEY_HINT;
    }
}

impl Widget for &Keyboard {
    fn render(self, block_area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // Render the surrounding block
        let title = Line::from(format!(" Layout - {} ", self.layout.name).bold());
        let instructions = Line::from(vec![
            " Toggle Hints ".into(),
            "<C-h> ".blue().bold(),
            " Next Layout ".into(),
            "<C-n> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .dark_gray()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::ROUNDED);
        let keyboard_area = block.inner(block_area);
        block.render(block_area, buf);

        // Get the vertical layout for the keyboard
        let rows_num = self.layout.base.len();
        let mut row_height = keyboard_area.height / rows_num as u16;
        if row_height % 2 == 0 {
            row_height -= 1;
        }
        let row_layout = {
            let mut constraints = vec![];
            constraints.push(Constraint::Fill(1));
            constraints.extend(std::iter::repeat_n(
                Constraint::Length(row_height),
                rows_num,
            ));
            constraints.push(Constraint::Length(1));
            constraints.push(Constraint::Fill(1));
            TuiLayout::vertical(constraints).split(keyboard_area)
        };

        // Get the horizontal layout
        let cols_num = self.layout.base.iter().map(|row| row.len()).max().unwrap();
        let mut col_width = keyboard_area.width / cols_num as u16;
        if col_width % 2 == 0 {
            col_width -= 1;
        }
        let col_constraints = {
            let mut constraints = vec![];
            constraints.push(Constraint::Fill(1));
            constraints.extend(std::iter::repeat_n(Constraint::Length(col_width), cols_num));
            constraints.push(Constraint::Fill(1));
            constraints
        };

        // Render the rows
        let mut row_areas = row_layout.iter();
        for (row_area, row) in (&mut row_areas).skip(1).zip(&self.keys).take(rows_num) {
            let key_layout = { TuiLayout::horizontal(col_constraints.clone()).split(*row_area) };
            for (key_area, key) in key_layout.iter().skip(1).zip(row).take(cols_num) {
                key.render(*key_area, buf);
            }
        }
        let modifier_row = row_areas.next().unwrap();
        let cur_width = self.cur.text.width() + 2;
        let sym_width = self.sym.text.width() + 2;
        let shift_width = self.shift.text.width() + 2;
        let [_, cur, sym, shift, _] = TuiLayout::horizontal([
            Constraint::Fill(1),
            Constraint::Max(cur_width as u16),
            Constraint::Max(sym_width as u16),
            Constraint::Max(shift_width as u16),
            Constraint::Fill(1),
        ])
        .areas(*modifier_row);
        self.cur.render(cur, buf);
        self.sym.render(sym, buf);
        self.shift.render(shift, buf);
    }
}

pub struct App {
    keyboard: Keyboard,
    file_data: FileData,
    exit: bool,
}

impl App {
    pub fn next(&self) -> Option<char> {
        self.file_data
            .story
            .chars()
            .nth(self.file_data.progress.chars)
    }
    pub fn load() -> Self {
        Self {
            keyboard: Keyboard::default(),
            file_data: FileData::load().unwrap(),
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        self.file_data.save().unwrap();
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        if self.keyboard.draw {
            let vertical = TuiLayout::vertical([Constraint::Fill(2), Constraint::Fill(1)]);
            let [app, keyboard] = vertical.areas(frame.area());
            // Update the highlighted block for the keyboard
            if let Some(c) = self.next() {
                self.keyboard.update(c);
            }
            frame.render_widget(&self.keyboard, keyboard);
            frame.render_widget(&*self, app);
        } else {
            frame.render_widget(&*self, frame.area());
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event {
            KeyEvent {
                code: KeyCode::Esc, ..
            } => self.exit(),
            KeyEvent {
                code: KeyCode::Char('n'),
                modifiers,
                ..
            } if modifiers.contains(KeyModifiers::CONTROL) => self.keyboard.next_layout(),
            KeyEvent {
                code: KeyCode::Char('h'),
                modifiers,
                ..
            } if modifiers.contains(KeyModifiers::CONTROL) => self.keyboard.toggle_draw(),
            KeyEvent {
                code: KeyCode::Char(char),
                ..
            } => {
                if self.next() == Some(char) {
                    self.file_data.progress.chars += 1;
                }
            }
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => {
                if self.next() == Some('↩') {
                    self.file_data.progress.chars += 1;
                }
            }
            KeyEvent {
                code: KeyCode::Tab, ..
            } => {
                self.file_data.progress.chars += 1;
            }
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, block_area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Story ".bold());
        let instructions = Line::from(vec![" Exit ".into(), "<Esc> ".blue().bold()]);
        let block = Block::bordered()
            .dark_gray()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::ROUNDED);
        let buff_width = block_area.width as usize / 3;
        let mut story = self
            .file_data
            .story
            .chars()
            .skip(self.file_data.progress.chars.saturating_sub(buff_width));
        let prefix_len = self.file_data.progress.chars
            - self.file_data.progress.chars.saturating_sub(buff_width);
        let prefix = (&mut story).take(prefix_len).collect::<String>();
        let current = (&mut story).take(1).collect::<String>();
        let postfix_len = (2 * buff_width).saturating_sub(prefix_len);
        let postfix = story.take(postfix_len).collect::<String>();
        let counter_text = Text::from(vec![Line::from(vec![
            prefix.dark_gray(),
            current.white().underlined().bold(),
            postfix.gray(),
        ])]);
        let area = block.inner(block_area);
        block.render(block_area, buf);
        let [_, area, _] = TuiLayout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(area);
        Paragraph::new(counter_text).centered().render(area, buf);
    }
}

type Layer = &'static [&'static [char]];

enum Modifier {
    Shift,
    Sym,
    Cur,
}

struct Location {
    row: u8,
    col: u8,
    modifier: Option<Modifier>,
}

struct Layout {
    name: &'static str,
    base: Layer,
    sym: Layer,
    cur: Layer,
}

impl Layout {
    fn shift(c: char) -> char {
        match c {
            '`' => '~',
            '1' => '!',
            '2' => '@',
            '3' => '#',
            '4' => '$',
            '5' => '%',
            '6' => '^',
            '7' => '&',
            '8' => '*',
            '9' => '(',
            '0' => ')',
            '[' => '{',
            ']' => '}',
            '\'' => '"',
            ',' => '<',
            '.' => '>',
            '/' => '?',
            '=' => '+',
            '\\' => '|',
            '-' => '_',
            ';' => ':',
            c => c.to_ascii_uppercase(),
        }
    }
    fn location(&self, c: char) -> Option<Location> {
        // Check the base layer
        for (row_i, row) in self.base.iter().enumerate() {
            for (col_i, c_candidate) in row.iter().enumerate() {
                if *c_candidate == c {
                    return Some(Location {
                        row: row_i as u8,
                        col: col_i as u8,
                        modifier: None,
                    });
                }
            }
        }
        // Check the sym layer
        for (row_i, row) in self.sym.iter().enumerate() {
            for (col_i, c_candidate) in row.iter().enumerate() {
                if *c_candidate == c {
                    return Some(Location {
                        row: row_i as u8,
                        col: col_i as u8,
                        modifier: Some(Modifier::Sym),
                    });
                }
            }
        }
        // Check the cur layer
        for (row_i, row) in self.cur.iter().enumerate() {
            for (col_i, c_candidate) in row.iter().enumerate() {
                if *c_candidate == c {
                    return Some(Location {
                        row: row_i as u8,
                        col: col_i as u8 + 6,
                        modifier: Some(Modifier::Cur),
                    });
                }
            }
        }
        // Check the shifted base layer
        for (row_i, row) in self.base.iter().enumerate() {
            for (col_i, c_candidate) in row.iter().enumerate() {
                if Layout::shift(*c_candidate) == c {
                    return Some(Location {
                        row: row_i as u8,
                        col: col_i as u8,
                        modifier: Some(Modifier::Shift),
                    });
                }
            }
        }
        None
    }
}

const LAYOUT_QWERTY: Layout = Layout {
    name: "QWERTY",
    base: KEYS_QWERTY_BASE,
    sym: &[],
    cur: &[],
};

const KEYS_QWERTY_BASE: &[&[char]] = &[
    &[
        '`', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '[', ']', '\0',
    ],
    &[
        '\0', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\\',
    ],
    &[
        '\0', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '\0', '\0',
    ],
    &[
        '\0', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/', '\0', '\0', '\0',
    ],
];

const LAYOUT_DVORAK: Layout = Layout {
    name: "Dvorak",
    base: KEYS_DVORAK_BASE,
    sym: &[],
    cur: &[],
};

const KEYS_DVORAK_BASE: &[&[char]] = &[
    &[
        '`', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '[', ']', '\0',
    ],
    &[
        '\0', '\'', ',', '.', 'p', 'y', 'f', 'g', 'c', 'r', '/', '=', '\\', '\0',
    ],
    &[
        '\0', 'a', 'o', 'e', 'u', 'i', 'd', 'h', 't', 'n', 's', '-', '\0', '\0',
    ],
    &[
        '\0', ';', 'q', 'j', 'k', 'x', 'b', 'm', 'w', 'v', 'z', '\0', '\0', '\0',
    ],
];

const LAYOUT_3L: Layout = Layout {
    name: "3l",
    base: KEYS_3L_BASE,
    sym: KEYS_3L_SYM,
    cur: KEYS_3L_CUR,
};

const KEYS_3L_BASE: &[&[char]] = &[
    &['q', 'f', 'u', 'y', 'z', 'x', 'k', 'c', 'w', 'b'],
    &['o', 'h', 'e', 'a', 'i', 'd', 'r', 't', 'n', 's'],
    &[',', 'm', '.', 'j', ';', 'g', 'l', 'p', 'v', '\0'],
];

const KEYS_3L_SYM: &[&[char]] = &[
    &['"', '_', '[', ']', '^', '!', '<', '>', '=', '&'],
    &['/', '-', '{', '}', '*', '?', '(', ')', '\'', ':'],
    &['#', '$', '|', '~', '`', '+', '%', '\\', '@'],
];
const KEYS_3L_CUR: &[&[char]] = &[
    &['\0', '1', '2', '3'],
    &['\0', '4', '5', '6'],
    &['0', '7', '8', '9'],
];
