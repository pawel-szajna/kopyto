# kopyto

Chess engine experiment which perhaps is now slowly turning out maybe to
be just a chess  engine. It communicates using the UCI interface. It also
has a [Lichess account](https://lichess.org/@/kopyto_dev) where you can
challenge it.

## Getting it running

Clone the repo, then run:

```shell
$ RUSTFLAGS='-C target-cpu=native' cargo build --release
```

A binary should appear in `target/release` directory.

## Boring technical stuff

* Board:
    * (6+1)×2+1 Bitboards (every piece type per side, occupancy per
      side, any side occupancy)
    * Legal move generator using magic bitboards
    * Zobrist hashing
* Search:
    * Alpha-beta search with zero window
    * Iterative deepening
    * Transposition table
    * Move ordering:
        * PV move
        * MVV-LVA
        * killer moves
        * history heuristic
    * Check extensions (12.0±21.1)
    * Null move pruning (93.2±27.6)
    * Reverse futility pruning (26.1±21.6)
    * Mate distance pruning
    * Delta pruning
    * Razoring
    * Built-in simple opening book (optional, disabled by default)
* Hand-crafted evaluation function:
    * Piece-square tables (91.5±32.9)
    * Simple mobility bonus (105.0±35.3)
    * Doubled/isolated pawn penalties (20.0±27.0)
    * ~~King position bonuses~~ (-43.3±25.0)
    * Tapered evaluation

The ELO change measurements are results of several hundreds of games
against a build without this feature. They may come from various stages
of engine development, and probably don't really mean much.
