use super::CommandHandler;

wit_bindgen::generate!({
    path: "../../wit",
    world: "command-handler-w",
});

export!(CommandHandler);
