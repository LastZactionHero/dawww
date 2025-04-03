use crate::pitch::Pitch;

pub mod score_draw_component;
pub mod status_bar_component;

#[derive(Clone, Copy)]
pub struct ViewportDrawResult {
    pub pitch_low: Pitch,
    pub pitch_high: Pitch,
    pub time_point_start: u64, // Inclusive
    pub time_point_end: u64,   // Exclusive
}

#[derive(Clone, Copy)]
pub enum DrawResult {
    ViewportDrawResult(ViewportDrawResult),
}

pub trait DrawComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) -> Vec<DrawResult>;

    fn wb(&self, buffer: &mut Vec<Vec<char>>, pos: &Position, x: usize, y: usize, value: char) {
        buffer[pos.y + y][pos.x + x] = value;
    }

    fn wb_string(
        &self,
        buffer: &mut Vec<Vec<char>>,
        pos: &Position,
        x: usize,
        y: usize,
        value: String,
    ) {
        for (i, char) in value.chars().enumerate() {
            if pos.x + x + i >= buffer[pos.y].len() {
                break;
            }
            buffer[pos.y + y][pos.x + x + i] = char;
        }
    }
}

pub struct Position {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl Position {
    fn right(&self) -> usize {
        self.x + self.w - 1
    }

    fn bottom(&self) -> usize {
        self.y + self.h - 1
    }
}

const BOX_TOP_LEFT: char = '╔';
const BOX_TOP_RIGHT: char = '╗';
const BOX_BOTTOM_LEFT: char = '╚';
const BOX_BOTTOM_RIGHT: char = '╝';
const BOX_HORIZONTAL: char = '═';
const BOX_VERTICAL: char = '║';
const BOX_RIGHT_DIVIDER: char = '╣';
const BOX_LEFT_DIVIDER: char = '╠';

pub struct Window {
    components: Vec<Box<dyn DrawComponent>>,
}

impl Window {
    pub fn new(components: Vec<Box<dyn DrawComponent>>) -> Window {
        Window { components }
    }
}

impl DrawComponent for Window {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) -> Vec<DrawResult> {
        let mut results = vec![];
        for component in &self.components {
            results.append(component.draw(buffer, &pos).as_mut());
        }
        return results;
    }
}

pub struct BoxDrawComponent {
    component: Box<dyn DrawComponent>,
}

impl BoxDrawComponent {
    pub fn new(component: Box<dyn DrawComponent>) -> BoxDrawComponent {
        BoxDrawComponent { component }
    }
}

impl DrawComponent for BoxDrawComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) -> Vec<DrawResult> {
        for x in pos.x + 1..pos.right() {
            self.wb(buffer, pos, x, 0, BOX_HORIZONTAL);
            self.wb(buffer, pos, x, pos.h - 1, BOX_HORIZONTAL);
        }
        for y in pos.y + 1..pos.bottom() {
            buffer[y][0] = BOX_VERTICAL;
            buffer[y][pos.x + pos.w - 1] = BOX_VERTICAL;
        }
        self.wb(buffer, pos, pos.x, pos.y, BOX_TOP_LEFT);
        self.wb(buffer, pos, pos.right(), pos.y, BOX_TOP_RIGHT);
        self.wb(buffer, pos, pos.x, pos.bottom(), BOX_BOTTOM_LEFT);
        self.wb(buffer, pos, pos.right(), pos.bottom(), BOX_BOTTOM_RIGHT);

        return self.component.draw(buffer, pos);
    }
}

pub struct VSplitDrawComponent {
    style: VSplitStyle,
    top_component: Box<dyn DrawComponent>,
    bottom_component: Box<dyn DrawComponent>,
}

#[derive(PartialEq, Eq)]
pub enum VSplitStyle {
    HalfWithDivider,
    StatusBarNoDivider,
}

impl VSplitDrawComponent {
    pub fn new(
        style: VSplitStyle,
        top_component: Box<dyn DrawComponent>,
        bottom_component: Box<dyn DrawComponent>,
    ) -> VSplitDrawComponent {
        VSplitDrawComponent {
            style,
            top_component,
            bottom_component,
        }
    }
}

impl DrawComponent for VSplitDrawComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) -> Vec<DrawResult> {
        let pos_top = match self.style {
            VSplitStyle::HalfWithDivider => Position {
                x: pos.x + 1,
                y: pos.y + 1,
                w: pos.w - 2,
                h: pos.h / 2,
            },
            VSplitStyle::StatusBarNoDivider => Position {
                x: pos.x,
                y: pos.y,
                w: pos.w,
                h: pos.h - 1,
            },
        };

        let pos_bottom = match self.style {
            VSplitStyle::HalfWithDivider => Position {
                x: pos.x + 1,
                y: pos.y + pos.h / 2 + 2,
                w: pos.w - 2,
                h: pos.h / 2 - 2,
            },
            VSplitStyle::StatusBarNoDivider => Position {
                x: pos.x,
                y: pos.y + pos.h - 1,
                w: pos.w,
                h: 1,
            },
        };

        let mut result = vec![];
        result.append(self.top_component.draw(buffer, &pos_top).as_mut());
        result.append(self.bottom_component.draw(buffer, &pos_bottom).as_mut());

        if self.style == VSplitStyle::HalfWithDivider {
            for x in 1..pos.w - 1 {
                self.wb(buffer, pos, x, pos.h / 2 + 1, '═');
            }
            self.wb(buffer, pos, 0, pos.h / 2 + 1, BOX_LEFT_DIVIDER);
            self.wb(buffer, pos, pos.w - 1, pos.h / 2 + 1, BOX_RIGHT_DIVIDER);
        }

        result
    }
}

pub struct NullComponent {}

impl DrawComponent for NullComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) -> Vec<DrawResult> {
        vec![]
    }
}

pub struct FillComponent {
    pub value: char,
}

impl DrawComponent for FillComponent {
    fn draw(&self, buffer: &mut Vec<Vec<char>>, pos: &Position) -> Vec<DrawResult> {
        for x in 0..pos.w {
            for y in 0..pos.h {
                self.wb(buffer, pos, x, y, self.value);
            }
        }
        vec![]
    }
}
