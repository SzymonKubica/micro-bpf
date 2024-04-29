import matplotlib.pyplot as plt


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


def main():

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

    message_sizes, _ = zip(*extended_interpreter_results);

    plt.plot(message_sizes, list(zip(*extended_interpreter_results))[1])
    plt.scatter(message_sizes, list(zip(*extended_interpreter_results))[1], label="Extended Interpreter")
    plt.plot(message_sizes, list(zip(*fc_interpreter_results))[1])
    plt.scatter(message_sizes, list(zip(*fc_interpreter_results))[1], label="Femto-Containers Interpreter")
    plt.plot(message_sizes, list(zip(*raw_object_file_results))[1])
    plt.scatter(message_sizes, list(zip(*raw_object_file_results))[1], label="Raw Object File Interpreter")
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


if __name__ == "__main__":
    main()
