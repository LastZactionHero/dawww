use crate::score::Score;

#[derive(Debug, Clone)]
pub enum SelectionBuffer {
    None,
    Score(Score),
}

impl SelectionBuffer {
    pub fn translate_to(&self, time_point_start_b32: u64) -> SelectionBuffer {
        match self {
            SelectionBuffer::None => self.clone(),
            SelectionBuffer::Score(score) => {
                let translated_score = score.translate(Some(time_point_start_b32));
                SelectionBuffer::Score(translated_score)
            }
        }
    }
}
