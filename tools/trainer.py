#!/usr/bin/env python3

import chess
import sys
import torch
import torch.nn as nn
import torch.optim as optim
import torch.utils.data as data
from tqdm import tqdm


EPOCHS = 100

CHESS_BOARD_SIZE = 64
CHESS_PIECE_TYPES = 6
CHESS_SIDES_COUNT = 2

INPUT_SIZE = CHESS_BOARD_SIZE * CHESS_PIECE_TYPES * CHESS_SIDES_COUNT


def clipped_relu(data):
    return torch.clamp(data, 0, 1)


def clipped_score(data):
    return torch.clamp(data, -1, 1)


class ChessDataset(data.Dataset):
    def __init__(self, data_file):
        self.data = []
        src = []
        with open(data_file, "r") as handle:
            src = [line for line in handle.readlines()]
        for line in tqdm(src, "Preparing data"):
            (input, output) = line.rsplit(";", 1)
            expected_score = float(output) / 10000
            self.data.append((fen_to_position(input), torch.Tensor([expected_score])))

    def __len__(self):
        return len(self.data)

    def __getitem__(self, index):
        return self.data[index]
        # (input, output) = self.data[index].rsplit(" ", 1)
        # return


class NN(nn.Module):
    def __init__(self):
        super(NN, self).__init__()
        self.fc1 = nn.Linear(INPUT_SIZE, 64)
        self.fc2 = nn.Linear(64, 32)
        self.fc3 = nn.Linear(32, 1)

    def print_position(self, position):
        ones = []
        for i, value in enumerate(position):
            if value > 0:
                ones.append(i)
        print(ones)
        pass

    def forward(self, position):
        # self.print_position(position)
        l1_out = self.fc1(position)
        # print(l1_out)
        l1_act = clipped_relu(l1_out)
        # print(l1_act)
        l2_out = self.fc2(l1_act)
        # print(l2_out)
        l2_act = clipped_relu(l2_out)
        # print(l2_act)
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

        # print(f"square: {square}, piece type: {piece.piece_type}, piece color: {piece.color}, piece_mod: {piece_mod}, color_mod: {color_mod}, feature_idx: {feature_idx}")

        in_layer[feature_idx] = 1

    return in_layer


if "--rate" in sys.argv:
    position = sys.argv[2]
    print(f"Rating: {position}")
    network = NN()
    network.to("cuda")
    network.load_state_dict(torch.load(f"net/network_e{EPOCHS}.pickle", weights_only=True))
    ev = network(fen_to_position(position).to("cuda"))
    print(f"Eval: {round(ev.item() / 100, 2)}")
    sys.exit(0)

if "--export" in sys.argv:
    SPACING = ' ' * 4
    network = NN()
    network.load_state_dict(torch.load(f"net/network_e{EPOCHS}.pickle", weights_only=True))

    params = list(network.parameters())

    print(len(params))
    indent = 1

    def quantize(tensor):
        # return tensor.item()
        return max(-127, min(127, int(round(tensor.item() * 127, 0))))

    def export_vector(vec):
        global indent
        indent += 1
        print(f"{indent * SPACING}{len(vec)}")
        print(f"{indent * SPACING}{' '.join([f'{quantize(w)}' for w in vec])}")
        indent -= 1
        pass

    for param in params:
        dims = param.dim()
        print(f"{indent * SPACING}{dims}")
        if dims == 1:
            indent += 1
            export_vector(param)
            indent -= 1
        elif dims == 2:
            indent += 1
            print(f"{indent * SPACING}{len(param)}")
            for v in param:
                export_vector(v)
            indent -= 1
        else:
            raise f"unexpected dimension: {dims}"

    sys.exit(0)

fen = "8/2k2p2/2p5/4bP1P/4P3/7q/2Q5/3K4 w - - 31 68"

print(chess.Board(fen))
network = NN()
network.to("cuda")
print(network(fen_to_position(fen).to("cuda")))

criterion = nn.MSELoss()

dataset = ChessDataset("./training_data.txt")
loader = data.DataLoader(dataset, 128)

lr = 3e-2

batches_per_epoch = len(loader)

with tqdm(total=EPOCHS * batches_per_epoch) as progress:
    for epoch in range(EPOCHS):
        lr *= 0.99
        loss = 0

        network.train(True)
        optimizer = optim.Adam(network.parameters())

        for i, data in enumerate(loader):
            inputs, evals = data
            optimizer.zero_grad()
            outputs = network(inputs.to("cuda"))
            loss = criterion(outputs, evals.to("cuda"))
            loss.backward()
            optimizer.step()
            if i % 10 == 0:
                progress.set_description_str(f"Epoch {epoch+1}/{EPOCHS}  LR {lr}  Loss {round(loss.item(), 5)}")
                progress.update(10)

        torch.save(network.state_dict(), f"net/network_e{epoch+1}.pickle")

print(network(fen_to_position(fen).to("cuda")))
torch.save(network.state_dict(), "net/network.pickle")
