pub struct SDLDisplayParameters {
    pub output_size:Option<(u32,u32)>,
    pub offset:Option<(i32,i32)>
}

impl SDLDisplayParameters {
    pub fn new(output_size:Option<(u32,u32)>,
               offset:Option<(i32,i32)>) -> Self {
        SDLDisplayParameters {
            output_size:output_size,
            offset:offset
        }
    }
}
