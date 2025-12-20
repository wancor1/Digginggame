pub enum GameEvent {
    StartGame, // Just kept for compatibility or used as "Game Started" signal
    OpenSaveSelection,
    LoadSave(String),
    StartNewGameSetup,
    ConfirmNewGame(String),
    SaveGame,
    QuitGame,
    ReturnToTitle,
    ReturnToTitleFromSaveSelect,
    // Add more UI events as needed
}

pub enum CameraMoveIntent {
    Up,
    Down,
    Left,
    Right,
    None,
}
