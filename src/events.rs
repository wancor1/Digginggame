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
    UpgradeDrill,
    UpgradeTank,
    UpgradeEngine,
    UpgradeCargo,
    BuyWarpGate,
    StartPlaceWarpGate,
    ConfirmWarpGateName(String),
    OpenWarpMenu,
    TeleportToWarp(usize),
    CloseMenu,
}

pub enum CameraMoveIntent {
    Up,
    Down,
    Left,
    Right,
    None,
}
