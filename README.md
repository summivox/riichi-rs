# Crates for Japanese Riichi Mahjong

This repo hosts crates related to (Standard) [Japanese Riichi Mahjong][riichi-wiki], a card game played with tiles on a 
table.

## Table of Contents

- [riichi](./riichi): Game engine library.
  - [riichi-elements](./riichi-elements): Building blocks of the game.
  - [riichi-decomp](./riichi-decomp): Waiting hand decomposition.
    - [riichi-decomp-table](./riichi-decomp-table): Look-up table (LUT) for `riichi-decomp`.

- [tenhou-db](./tenhou-db): Download public game logs/replays from [Tenhou] and organize them into a SQLite database.
  - [tenhou-shuffle](./tenhou-shuffle): Independent (re-)implementation of [Tenhou]'s wall/deck-shuffling algorithm.

[riichi-wiki]: https://riichi.wiki/Japanese_mahjong
[Tenhou]: https://tenhou.net
