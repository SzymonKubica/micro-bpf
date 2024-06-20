import pandas as pd
import matplotlib.pyplot as plt
from collections import defaultdict


def old_bench():
    results = open("results2.txt", "r")

    message_sizes = []
    rbpf_times = []
    femtocontainer_times = []

    for i, line in enumerate(results.read().strip().split("\n")):
        parts = line.split(" ")
        rbpf_time = int(parts[1].rstrip(parts[1][-1]))
        femtocontainer_time = int(parts[3].rstrip(parts[3][-1]))
        rbpf_times.append(rbpf_time)
        femtocontainer_times.append(femtocontainer_time)
        message_sizes.append(128 * (i + 1))

    print(message_sizes)
    print(rbpf_times)
    print(femtocontainer_times)
    plt.plot(message_sizes, rbpf_times, label="rBPF")
    plt.scatter(
        message_sizes,
        rbpf_times,
    )
    plt.plot(message_sizes, femtocontainer_times, label="Femto-Containers")
    plt.scatter(message_sizes, femtocontainer_times)
    plt.xticks(message_sizes)
    plt.xlabel("Message Size [B]")
    plt.ylabel("Execution Time [us]")
    plt.legend()
    plt.show()


def plots_benchmark2():

    extended_interpreter_results = [
        (80, 728),
        (160, 1435),
        (320, 2849),
        (640, 5674),
        (1280, 11328),
        (2560, 22634),
    ]

    fc_interpreter_results = [
        (80, 323),
        (160, 641),
        (320, 1275),
        (640, 2545),
        (1280, 5083),
        (2560, 10159),
    ]

    jit_results = [(80, 18), (160, 36), (320, 70), (640, 139), (1280, 278), (2560, 554)]

    native_results = [
        (80, 15),
        (160, 29),
        (320, 57),
        (640, 114),
        (1280, 226),
        (2560, 451),
    ]

    raw_object_file_results = [
        (80, 1053),
        (160, 1988),
        (320, 3860),
        (640, 7604),
        (1280, 15092),
        (2560, 30066),
    ]

    message_sizes, _ = zip(*extended_interpreter_results)

    plt.plot(message_sizes, list(zip(*extended_interpreter_results))[1])
    plt.scatter(
        message_sizes,
        list(zip(*extended_interpreter_results))[1],
        label="Extended Interpreter",
    )
    plt.plot(message_sizes, list(zip(*fc_interpreter_results))[1])
    plt.scatter(
        message_sizes,
        list(zip(*fc_interpreter_results))[1],
        label="Femto-Containers Interpreter",
    )
    plt.plot(message_sizes, list(zip(*raw_object_file_results))[1])
    plt.scatter(
        message_sizes,
        list(zip(*raw_object_file_results))[1],
        label="Raw Object File Interpreter",
    )
    plt.plot(message_sizes, list(zip(*native_results))[1])
    plt.scatter(message_sizes, list(zip(*native_results))[1], label="Native")
    plt.plot(message_sizes, list(zip(*jit_results))[1])
    plt.scatter(message_sizes, list(zip(*jit_results))[1], label="JIT")
    plt.xticks(message_sizes)
    plt.xlabel("Message Size [B]")
    plt.ylabel("Execution Time [us]")
    plt.xscale("log")
    plt.legend()
    plt.show()


def plots_benchmark_overhead_investigation():

    extended_interpreter_results = [
        (80, 46),
        (160, 35),
        (320, 50),
        (640, 73),
        (1280, 169),
        (2560, 227),
    ]

    fc_interpreter_results = [
        (80, 7),
        (160, 12),
        (320, 17),
        (640, 28),
        (1280, 80),
        (2560, 135),
    ]

    raw_object_file_results = [
        (80, 23),
        (160, 36),
        (320, 45),
        (640, 158),
        (1280, 140),
        (2560, 266),
    ]

    message_sizes, _ = zip(*extended_interpreter_results)
    print(message_sizes)

    plt.plot(message_sizes, list(zip(*extended_interpreter_results))[1])
    plt.scatter(
        message_sizes,
        list(zip(*extended_interpreter_results))[1],
        label="Extended Interpreter",
    )
    plt.plot(message_sizes, list(zip(*fc_interpreter_results))[1])
    plt.scatter(
        message_sizes,
        list(zip(*fc_interpreter_results))[1],
        label="Femto-Containers Interpreter",
    )
    plt.plot(message_sizes, list(zip(*raw_object_file_results))[1])
    plt.scatter(
        message_sizes,
        list(zip(*raw_object_file_results))[1],
        label="Raw Object File Interpreter",
    )
    plt.xlabel("Message Size [B]")
    plt.ylabel("Execution Time [us]")
    plt.xticks(message_sizes, message_sizes)
    plt.xscale("log")
    plt.legend()
    plt.show()


def plots_examples():
    fc_results = [
        {"reloc": 1, "load": 0, "verif": 3, "exec": 4099, "prog": 162, "result": 0},
        {"reloc": 1, "load": 0, "verif": 1, "exec": 8815, "prog": 274, "result": 5},
        {"reloc": 1, "load": 0, "verif": 1, "exec": 8810, "prog": 250, "result": 5},
        {"reloc": 1, "load": 1, "verif": 1, "exec": 10728, "prog": 338, "result": 1234},
        {"reloc": 1, "load": 1, "verif": 1, "exec": 4576, "prog": 602, "result": 32742},
    ]

    raw_object_file_results = [
        {"reloc": 91, "load": 136, "verif": 90, "exec": 934, "prog": 1016, "result": 0},
        {
            "reloc": 112,
            "load": 139,
            "verif": 93,
            "exec": 5648,
            "prog": 1200,
            "result": 5,
        },
        {
            "reloc": 109,
            "load": 136,
            "verif": 92,
            "exec": 5648,
            "prog": 1176,
            "result": 5,
        },
        {
            "reloc": 100,
            "load": 138,
            "verif": 101,
            "exec": 7824,
            "prog": 1320,
            "result": 1234,
        },
        {
            "reloc": 101,
            "load": 135,
            "verif": 109,
            "exec": 3014,
            "prog": 1544,
            "result": 32742,
        },
    ]

    fc_header_results = [
        {"reloc": 0, "load": 137, "verif": 9, "exec": 827, "prog": 162, "result": 0},
        {"reloc": 1, "load": 136, "verif": 13, "exec": 5534, "prog": 274, "result": 5},
        {"reloc": 1, "load": 138, "verif": 12, "exec": 5539, "prog": 250, "result": 5},
        {
            "reloc": 0,
            "load": 135,
            "verif": 16,
            "exec": 7717,
            "prog": 338,
            "result": 1234,
        },
        {
            "reloc": 0,
            "load": 137,
            "verif": 20,
            "exec": 2912,
            "prog": 602,
            "result": 32742,
        },
    ]

    extended_header_results = [
        {"reloc": 1, "load": 136, "verif": 9, "exec": 840, "prog": 194, "result": 0},
        {"reloc": 1, "load": 138, "verif": 13, "exec": 5553, "prog": 306, "result": 5},
        {"reloc": 1, "load": 135, "verif": 11, "exec": 5555, "prog": 282, "result": 5},
        {
            "reloc": 0,
            "load": 138,
            "verif": 16,
            "exec": 7739,
            "prog": 370,
            "result": 1234,
        },
        {
            "reloc": 1,
            "load": 136,
            "verif": 21,
            "exec": 2927,
            "prog": 634,
            "result": 32742,
        },
    ]

    only_text_section_results = [
        {"reloc": 1, "load": 138, "verif": 13, "exec": 824, "prog": 152, "result": 0},
        {"reloc": 0, "load": 135, "verif": 28, "exec": 5552, "prog": 384, "result": 5},
        {"reloc": 1, "load": 138, "verif": 27, "exec": 5543, "prog": 368, "result": 5},
        {
            "reloc": 1,
            "load": 135,
            "verif": 33,
            "exec": 7733,
            "prog": 472,
            "result": 1234,
        },
        {"reloc": 0, "load": 137, "verif": 20, "exec": 16, "prog": 232, "result": 0},
    ]

    jit_results = [
        {
            "prog_size": 1016,
            "jit_prog_size": 100,
            "jit_comp_time": 307,
            "run_time": 3,
            "result": 0,
        },
        {
            "prog_size": 1200,
            "jit_prog_size": 124,
            "jit_comp_time": 341,
            "run_time": 5,
            "result": 134279611,
        },
        {
            "prog_size": 1176,
            "jit_prog_size": 120,
            "jit_comp_time": 343,
            "run_time": 4,
            "result": 5,
        },
        {
            "prog_size": 1320,
            "jit_prog_size": 200,
            "jit_comp_time": 385,
            "run_time": 7,
            "result": 1234,
        },
        {
            "prog_size": 1544,
            "jit_prog_size": 470,
            "jit_comp_time": 366,
            "run_time": 72,
            "result": 32742,
        },
    ]
    all_results = [
        raw_object_file_results,
        fc_results,
        fc_header_results,
        extended_header_results,
        only_text_section_results,
    ]
    columns = [
        "raw_object_file",
        "fc",
        "fc_header",
        "extended_header",
        "only_text_section",
    ]

    test_programs = [
        "bpf_fetch.c",
        "bpf_fmt_s16_dfp.c",
        "bpf_fmt_u32_dec.c",
        "bpf_store.c",
        "fletcher16",
    ]

    metrics = ["reloc", "load", "verif", "exec", "prog"]
    titles = {"reloc": "Relocation resolution time", "load": "Load time", "verif": "Verification time", "exec": "Execution time", "prog": "Program size"}
    for metric in metrics:
        data = defaultdict(lambda: [])
        for i, column in enumerate(columns):
            for j, prog in enumerate(test_programs):
                data[column].append(all_results[i][j][metric])

        _df = pd.DataFrame(data, columns=columns, index=test_programs)
        _df.plot.bar(title=titles[metric])


    plt.show()


if __name__ == "__main__":
    plots_examples()
