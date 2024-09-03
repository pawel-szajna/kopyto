# kopyto

Chess engine experiment which perhaps is now slowly turning out maybe to
be just a chess  engine. It communicates using the UCI interface. It also
has a [Lichess account](https://lichess.org/@/kopyto_dev) where you can
challenge it.

## Change log

| Version (tag)      | Features                                                           | W–D–L vs. previous |
|--------------------|--------------------------------------------------------------------|--------------------|
| `v9_book`          | (small) openings book, performance tweaks, evaluation fixes        | 59–33–8            |
| `v8_zerowindow`    | zero window search, magic bitboards, time management upgrades      | 78–21–1            |
| `v7_iterdeepening` | iterative deepening, aspiration windows, minor fixes               | 28–21–12           |
| `v6_bettermovegen` | noticeable move generation speedup, depth increased                | 157–20–23          |
| `v5_quiescence`    | quiescence search, transposition tables, temporarily reduced depth | 99–131–70          |
| `v4_weighteval`    | weights for pieces position during evaluation                      | 157–69–30          |
| `v3_alphabeta`     | alpha-beta pruning, general speed optimizations, deeper search     | 382–10–8           |
| `v2_negamax`       | searches (shallowly and slowly) the game tree                      | 984–10–6           |
| `v1_simplest`      | selects the move (without search) with simple evaluation           | 910–87–3           |
| `v0_random`        | move generation is working, "search" by random move                | N/A                |
