#![allow(non_snake_case)]

pub struct States;
impl States {
    // Estado inicial del programa
    // y cuando finaliza una partida | [2024.08.31-17.27.38:105][866]LogShooterGameState: Match Ended: Completion State: ''. Winning Team: 'Blue' () |
    pub const NOT_IN_GAME: u8 = 0;
    // Cuando empieza una partida | [2024.08.31-18.33.07:287][277]LogGameFlowStateManager: Reconcile called with state: TransitionToInGame and new state: InGame. Changing state. |
    // y cuando termina y empieza una ronda | [2024.08.31-17.25.31:152][599]LogShooterGameState: Warning: AShooterGameState::OnRoundEnded for round '22' |
    pub const IN_GAME_PREPARING: u8 = 1;
    // Cuando en la ronda, la preparación ha terminado | [2024.08.31-18.36.09:234][254]LogShooterGameState: Warning: Gameplay started at local time 30.218750 (server time 30.292187) | Si el mensaje tiene un time de 0, no es válido (se produce al terminar una ronda y spawnear). Ej. real de no válido: [2024.08.31-18.39.35:655][913]LogShooterGameState: Warning: Gameplay started at local time 0.000000 (server time 0.000000)
    // y cuando un personaje pasa de muerto a vivo | [2024.08.31-19.11.00:337][867]LogSkeletalMesh: Warning: USkeletalMeshComponent::RecreateClothingActors : (CosmeticCharacterMesh3P) Smonk_PC_C_2147249944 |
    pub const IN_GAME_PLAYING: u8 = 2;
    // Cuando un personaje pasa de vivo a muerto | [2024.08.31-18.51.39:595][870]LogAresMinimapComponent: Warning: Found Compute Position override on: MinimapRangeIndicator. Setting Position source to custom. Please change this in the asset. |
    // pub const IN_GAME_DEAD: u8 = 3;
}