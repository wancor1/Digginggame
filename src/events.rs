pub enum GameEvent {
    StartGame,
    SaveGame,
    QuitGame,
    // Add more UI events as needed
}

pub enum CameraMoveIntent {
    Up,
    Down,
    Left,
    Right,
    None,
}
