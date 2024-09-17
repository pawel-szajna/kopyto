import chess
import torch
import torch.nn as nn
import torch.optim as optim


CHESS_BOARD_SIZE = 64
CHESS_PIECE_TYPES = 6
CHESS_SIDES_COUNT = 2

INPUT_SIZE = CHESS_BOARD_SIZE * CHESS_PIECE_TYPES * CHESS_SIDES_COUNT


def clipped_relu(data):
    return torch.clamp(data, 0, 1)


def clipped_score(data):
    return torch.clamp(data, -1, 1)


class NN(nn.Module):
    def __init__(self):
        super(NN, self).__init__()
        self.fc1 = nn.Linear(INPUT_SIZE, 32)
        self.fc2 = nn.Linear(32, 24)
        self.fc3 = nn.Linear(24, 1)

    def forward(self, position):
        l1_out = self.fc1(position)
        l1_act = clipped_relu(l1_out)
        l2_out = self.fc2(l1_act)
        l2_act = clipped_relu(l2_out)
        l3_out = self.fc3(l2_act)
        return clipped_score(l3_out)


def fen_to_position(fen):
    board = chess.Board(fen)
    in_layer = torch.zeros(INPUT_SIZE)

    for square, piece in board.piece_map().items():
        piece_id = piece.piece_type - 1
        color_id = int(piece.color)

        piece_mod = CHESS_BOARD_SIZE * piece_id
        color_mod = CHESS_BOARD_SIZE * CHESS_PIECE_TYPES * color_id

        feature_idx = square + piece_mod + color_mod

        # print(f"{square}: {piece} (type: {piece.piece_type - 1} color: {1 + int(piece.color)}) -> piece_mod: {piece_mod} color_mod: {color_mod} -> {feature_idx}")
        in_layer[feature_idx] = 1

    return in_layer


fen = "8/2k2p2/2p5/4bP1P/4P3/7q/2Q5/3K4 w - - 31 68"

print(chess.Board(fen))
n = NN()
print(n(fen_to_position(fen)))

criterion = nn.MSELoss()
optimizer = optim.SGD(n.parameters())

optimizer.zero_grad()
inputs = torch.stack((
        fen_to_position("8/5k1p/8/3K1pp1/8/Rp4P1/1P3P1P/8 b - - 1 36"),
        fen_to_position("6k1/2p3p1/5p2/3pp2p/8/1r1R1PP1/2N4P/7K b - - 0 38")
    ))
outputs = n(inputs)
loss = criterion(outputs, torch.Tensor([[1], [-1]]))
loss.backward()
optimizer.step()

print(n(fen_to_position(fen)))

for i, parameter in enumerate(n.parameters()):
    print(f"param {i} - size: {parameter.size()}")
