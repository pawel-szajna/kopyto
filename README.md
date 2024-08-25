# kopyto

Chess engine experiment

## Building

To use the UI, you need to enable `ui` flag (which in turn makes the application depend on more stuff than
usual). The UI is quite simple though (and allows for making illegal moves. Don't make any illegal moves
though. I won't help you when the chess police comes)

To use it as an engine, launch it from command line with `uci` parameter.

## Change log

| Version (tag)          | Features                                                               | Score vs. previous (W–D–L) |
| ---------------------- | ---------------------------------------------------------------------- | -------------------------- |
| `v1_simplest`          | selects the move (without search) with simple evaluation               | 910–87–3                   |
| `v0_random`            | move generation is working, "search" by random move                    | N/A                        |

