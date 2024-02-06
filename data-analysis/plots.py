import matplotlib.pyplot as plt


def main():
    results = open("results.txt", "r")

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
    plt.scatter(message_sizes, rbpf_times,)
    plt.plot(message_sizes, femtocontainer_times, label="Femto-Containers")
    plt.scatter(message_sizes, femtocontainer_times)
    plt.xticks(message_sizes)
    plt.xlabel("Message Size [B]")
    plt.ylabel("Execution Time [us]")
    plt.legend()
    plt.show()






if __name__ == "__main__":
    main()
