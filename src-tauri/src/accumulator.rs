use bytes::Bytes;

enum Content {
    STRING(String),
    BYTES(Bytes)
}
struct Template{
    role: Option<String>,
    r#type: Option<String>,
    format: Option<String>,
    content: Option<String>,
}

struct Accumulator{
    template: Template,
    message: Template
}

pub enum Chunk {
    BYTES(Bytes),
    DICT
}

impl Accumulator{
    pub fn accumulate(&mut self, chunk: Chunk) -> Option<Template>{
        match chunk {
            Chunk::DICT => {

            },
            Chunk::BYTES(bytes) => {
                if self.message.content.is_none() || self.message.content.is_some_and(|c| c != Content::BYTES){
                    self.message["content"] = Bytes::new();
                }
                self.message["content"] += bytes;
                return None
            },
        }
    }
}
