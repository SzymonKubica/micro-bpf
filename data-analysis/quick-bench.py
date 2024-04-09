import matplotlib.pyplot as plt
import numpy as np

data_set1 = [
{"load_time": 0, "execution_time": 856,"program_size": 294, "result": 0},
{"load_time": 0, "execution_time": 5561,"program_size": 406, "result": 5},
{"load_time": 0, "execution_time": 5570,"program_size": 382, "result": 5},
{"load_time": 0, "execution_time": 7746,"program_size": 470, "result": 1234},
{"load_time": 0, "execution_time": 2251,"program_size": 390, "result": 21},
{"load_time": 0, "execution_time": 10693,"program_size": 886, "result": 134222414},
{"load_time": 0, "execution_time": 15277,"program_size": 694, "result": 12345},
{"load_time": 1, "execution_time": 14263,"program_size": 654, "result": 0},
]

data_set2 = [
 {"load_time": 89, "execution_time": 893,"program_size": 1016, "result": 0},
 {"load_time": 105, "execution_time": 5598,"program_size": 1200, "result": 5},
 {"load_time": 107, "execution_time": 5598,"program_size": 1176, "result": 5},
 {"load_time": 94, "execution_time": 7782,"program_size": 1320, "result": 1234},
 {"load_time": 93, "execution_time": 2288,"program_size": 1200, "result": 21},
 {"load_time": 119, "execution_time": 10711,"program_size": 1904, "result": 134222414},
 {"load_time": 109, "execution_time": 15305,"program_size": 1552, "result": 12345},
 {"load_time": 107, "execution_time": 14282,"program_size": 1536, "result": 0},
]



load_times1 = [d["load_time"] for d in data_set1]
execution_times1 = [d["execution_time"] for d in data_set1]
program_sizes1 = [d["program_size"] for d in data_set1]

load_times2 = [d["load_time"] for d in data_set2]
execution_times2 = [d["execution_time"] for d in data_set2]
program_sizes2 = [d["program_size"] for d in data_set2]

# Creating subplots
fig, axs = plt.subplots(1, 3, figsize=(15, 5))

x_tick_labels = [
        "bpf_fetch.c",
        "bpf_fmt_s16_dfp.c",
        "bpf_fmt_u32_dec.c",
        "bpf_store.c",
        "bpf_strlen.c",
        "gcoap_response_format.c",
        "inlined_calls.c",
        "printf.c"]


# Plotting load times
axs[0].bar(np.arange(len(load_times1)), load_times1, label='Custom Header Binary', width=0.4)
axs[0].bar(np.arange(len(load_times2)) + 0.4, load_times2, label='Raw Elf File', width=0.4)
axs[0].set_xticks(np.arange(len(load_times1)) + 0.2)
axs[0].set_xticklabels(x_tick_labels, rotation=45)
axs[0].set_ylabel('Load Time')
axs[0].set_title('Load Time [us]')
axs[0].legend()

# Plotting execution times
axs[1].bar(np.arange(len(execution_times1)), execution_times1, label='Custom Header Binary', width=0.4)
axs[1].bar(np.arange(len(execution_times2)) + 0.4, execution_times2, label='Raw Elf File', width=0.4)
axs[1].set_xticks(np.arange(len(execution_times1)) + 0.2)
axs[1].set_xticklabels(x_tick_labels, rotation=45)
axs[1].set_ylabel('Execution Time')
axs[1].set_title('Execution Time [us]')
axs[1].legend()

# Plotting program sizes
axs[2].bar(np.arange(len(program_sizes1)), program_sizes1, label='Custom Header Binary', width=0.4)
axs[2].bar(np.arange(len(program_sizes2)) + 0.4, program_sizes2, label='Raw Elf File', width=0.4)
axs[2].set_xticks(np.arange(len(program_sizes1)) + 0.2)
axs[2].set_xticklabels(x_tick_labels, rotation=45)
axs[2].set_ylabel('Program Size')
axs[2].set_title('Program Size [Bytes]')
axs[2].legend()

plt.tight_layout()
plt.show()

