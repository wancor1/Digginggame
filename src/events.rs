pub enum GameEvent {
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


