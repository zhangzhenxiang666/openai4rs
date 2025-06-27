use super::types::CompletionChoice;

impl CompletionChoice {
    pub fn is_reasoning(&self) -> bool {
        self.reasoning.as_ref().is_some_and(|reas| !reas.is_empty())
    }

    pub fn get_reasoning_str(&self) -> &str {
        match self.reasoning.as_ref() {
            Some(reasoning) => reasoning.as_str(),
            None => "",
        }
    }

    pub fn get_text_str(&self) -> &str {
        self.text.as_str()
    }
}
