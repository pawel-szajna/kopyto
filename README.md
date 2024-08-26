# kopyto

Chess engine experiment

## Building

To use the UI, you need to enable `ui` flag (which in turn makes the application depend on more stuff than
usual). The UI is quite simple though (and allows for making illegal moves. Don't make any illegal moves
though. I won't help you when the chess police comes)

To use it as an engine, launch it from command line with `uci` parameter.

## Change log

| Version (tag)          | Features                                                               | Score vs. previous (W–D–L, win %) |
| ---------------------- | ---------------------------------------------------------------------- | --------------------------------- |
| `v3_alphabeta`         | alpha-beta pruning, general speed optimizations, deeper search         | 382–10–8 (95.5%)                  |
| `v2_negamax`           | searches (shallowly and slowly) the game tree                          | 984–10–6 (98.4%)                  |
| `v1_simplest`          | selects the move (without search) with simple evaluation               | 910–87–3 (91.0%)                  |
| `v0_random`            | move generation is working, "search" by random move                    | N/A                               |

