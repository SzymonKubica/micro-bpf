import sys
import json
import csv
from collections import defaultdict
from typing import Dict


def process_data(file_name: str):
    with open(file_name, "r") as f:
        data = json.load(f)
        pretty_json = json.dumps(data, indent=4)
        print(pretty_json)


RESULTS_RAW_DIR = "benchmark-results-raw"
RESULTS_PROCESSED_DIR = "benchmark-results-processed"

METRICS = [
    "load_time",
    "verification_time",
    "execution_time",
    "total_time",
    "program_size",
]


# this array controlls the sorting of the fields in the output csv files
PLATFORMS = ["native", "femtocontainers-header", "extended-header", "jit"]


def process_fletcher16(data_size: int):
    """
    Takes in the data size of the fletcher16 benchmark (between 80-2560B)
    and produces csv outputs for that datapoint. It produces a csv containing
    performance of all different solutions for a single metric.
    """

    results_per_metric = load_fletcher16_metrics_data(data_size)

    for metric, results in results_per_metric.items():
        metric_str = metric.replace("_", "-")
        file_name = (
            f"{RESULTS_PROCESSED_DIR}/fletcher16-{data_size}-{metric_str}-results.csv"
        )
        with open(file_name, "w") as f:
            writer = csv.DictWriter(f, fieldnames=["platform", metric])
            writer.writeheader()
            for platform in PLATFORMS:
                # in the native C case, we don't measure all metrics so we skip this entry in the CSV
                if platform == "native" and metric in [
                    "load_time",
                    "verification_time",
                    "program_size",
                ]:
                    continue
                # in case of native total time is the execution time
                if platform == "native" and metric == "total_time":
                    writer.writerow(
                        {
                            "platform": platform,
                            metric: results_per_metric["execution_time"][platform],
                        }
                    )
                    continue
                writer.writerow({"platform": platform, metric: results[platform]})

            # We need to append this dummy row at the  end because that's how
            # the latex csv parser works
            writer.writerow({"platform": 0, metric: 0})


def load_fletcher16_metrics_data(data_size: int) -> Dict[str, Dict[str, int]]:
    result_files = [
        "femtocontainers-header-fletcher-results.json",
        "extended-header-fletcher-results.json",
        "extended-header-fast-insn-fletcher-results.json",
        "extended-header-slow-insn-fletcher-results.json",
        "jit-fletcher-results.json",
        "native-fletcher-results.json",
    ]

    results_per_metric: Dict[str, Dict[str, int]] = defaultdict(lambda: {})
    for m in METRICS:
        for file in result_files:
            file_name = f"{RESULTS_RAW_DIR}/{file}"
            with open(file_name, "r") as f:
                data = json.load(f)
                vm_kind = file.replace("-fletcher-results.json", "")
                results_per_metric[m][vm_kind] = (
                    data[str(data_size)][m] if m in data[str(data_size)].keys() else 0
                )
    return results_per_metric


def process_jit_fletcher16_execution_time():
    """
    Shows the raw execution time of the JIT for the fletcher16 benchmark.
    """

    # Here we only do first 4 datapoins because later it grows too large and it
    # is no longer visible
    data_sizes = [80 * 2**i for i in range(4)]

    platforms = PLATFORMS + ["extended-header-fast-insn", "extended-header-slow-insn"]

    results_per_platform = defaultdict(lambda: {})
    for data_size in data_sizes:
        results_per_metric = load_fletcher16_metrics_data(data_size)
        for platform in platforms:
            results_per_platform[platform][data_size] = results_per_metric[
                "execution_time"
            ][platform]

    for name, data in results_per_platform.items():
        file_name = (
            f"{RESULTS_PROCESSED_DIR}/fletcher16-all-sizes-execution-time-{name}.csv"
        )
        with open(file_name, "w") as f:
            writer = csv.DictWriter(f, fieldnames=["data_size", "execution-time"])
            writer.writeheader()
            for data_size in data_sizes:
                writer.writerow(
                    {"data_size": data_size, "execution-time": data[data_size]}
                )
            # We need to append this dummy row at the  end because that's how
            # the latex csv parser works
            writer.writerow({"data_size": 0, "execution-time": 0})


def process_jit_fletcher16_amortized_cost():
    """
    The idea behind this analysis is to show how the relative jit compilation cost decreases
    with an increase in the program computation. It produces three csv files:
        - total fletcher16 execution time for Femto-Container baseline for varying fletcher16 data sizes
        - total execution for mibpf jit as above.
        - jit compilation time as above
        - jit execution time for datapoints as above
    """

    data_sizes = [80 * 2**i for i in range(6)]

    total_fc_times = {}
    total_jit_times = {}
    jit_comp_times = {}
    jit_exec_times = {}
    for data_size in data_sizes:
        results_per_metric = load_fletcher16_metrics_data(data_size)
        total_fc_times[data_size] = results_per_metric["total_time"][
            "femtocontainers-header"
        ]
        total_jit_times[data_size] = results_per_metric["total_time"]["jit"]
        jit_comp_times[data_size] = results_per_metric["load_time"]["jit"]
        jit_exec_times[data_size] = results_per_metric["execution_time"]["jit"]

    outputs = [
        ("femtocontainer-total-time", total_fc_times),
        ("jit-total-time", total_jit_times),
        ("jit-comp-time", jit_comp_times),
        ("jit-exec-time", jit_exec_times),
    ]

    for name, data in outputs:
        file_name = f"{RESULTS_PROCESSED_DIR}/fletcher16-all-sizes-{name}.csv"
        with open(file_name, "w") as f:
            writer = csv.DictWriter(f, fieldnames=["data_size", name])
            writer.writeheader()
            for data_size in data_sizes:
                writer.writerow({"data_size": data_size, name: data[data_size]})
            # We need to append this dummy row at the  end because that's how
            # the latex csv parser works
            writer.writerow({"data_size": 0, name: 0})


def process_program_sizes():
    """
    This analysis produces a breakdown of program sizes for a set of chosen
    example programs for each of the solutions.
    """

    example_programs = [
        "bpf_fetch.c",
        "bpf_store.c",
        "bpf_strlen.c",
        "bpf_fmt_s16_dfp.c",
        "bpf_fmt_u32_dec.c",
        "printf.c",
        "inlined_calls.c",
        "jit_fletcher16_checksum_320B_data.c",
        "sensor-processing.c",
        "sensor-processing-from-storage.c",
    ]

    platforms = [
        "femtocontainers-header",
        "extended-header",
        "raw-object-file",
        "text-section-only",
        "jit",
    ]

    result_files = [
        "femtocontainers-header-results.json",
        "extended-header-results.json",
        "raw-object-file-results.json",
        "only-text-section-results.json",
        "jit-results.json",
    ]

    results_per_platform = defaultdict(lambda: {})
    for platform, results_file in zip(platforms, result_files):
        with open(f"{RESULTS_RAW_DIR}/{results_file}") as f:
            data = json.load(f)
            for program in example_programs:
                # in case of only-text-section we use custom programs
                # with different names:
                if platform == "text-section-only" and program not in [
                    "jit_fletcher16_checksum_320B_data.c",
                    "sensor-processing.c",
                    "sensor-processing-from-storage.c",
                ]:
                    program = program.replace(".c", "_only_text.c")
                program_size = data[program]["program_size"]
                results_per_platform[platform][program] = program_size

    for platform, results in results_per_platform.items():
        file_name = f"{RESULTS_PROCESSED_DIR}/{platform}-example-program-sizes.csv"
        with open(file_name, "w") as f:
            writer = csv.DictWriter(f, fieldnames=["program", "program_size"])
            writer.writeheader()
            for program in example_programs:
                # in case of only-text-section we use custom programs
                # with different names:
                if platform == "text-section-only" and program not in [
                    "jit_fletcher16_checksum_320B_data.c",
                    "sensor-processing.c",
                    "sensor-processing-from-storage.c",
                ]:
                    program = program.replace(".c", "_only_text.c")
                writer.writerow({"program": program, "program_size": results[program]})
            # We need to append this dummy row at the  end because that's how
            # the latex csv parser works
            writer.writerow({"program": 0, "program_size": 0})


def process_memory_access_checks():
    allowed_regions_list_lengths = [1, 4, 8, 12, 16]
    region_types = ["stack_memory_access", "data_section_memory_access"]
    raw_result_files = [
        "memory-access-checks-cached.json",
        "memory-access-checks-no-cache.json",
    ]

    def load_json(file_name: str):
        file = f"{RESULTS_RAW_DIR}/{file_name}"
        with open(file, "r") as f:
            data = json.load(f)
            return data

    data_cached = load_json(raw_result_files[0])
    data_no_cache = load_json(raw_result_files[1])

    data = [(data_cached, "cache"), (data_no_cache, "no_cache")]
    for d, s in data:
        stack_access_datapoints = []
        data_access_datapoints = []
        for l in allowed_regions_list_lengths:
            stack_access_datapoints.append((l, d[region_types[0]][str(l)]["total_time"]))
            data_access_datapoints.append((l, d[region_types[1]][str(l)]["total_time"]))

        for t, d in zip(region_types, [stack_access_datapoints, data_access_datapoints]):
            file_name = f"{RESULTS_PROCESSED_DIR}/{t}_{s}.csv"
            csv_fieldnames = ["allowed-regions-length", "execution-time"]
            with open(file_name, "w") as f:
                writer = csv.DictWriter(f, fieldnames=csv_fieldnames)
                writer.writeheader()
                for l, t in d:
                    writer.writerow({csv_fieldnames[0]: l, csv_fieldnames[1]: t})
                # We need to append this dummy row at the  end because that's how
                # the latex csv parser works
                writer.writerow({csv_fieldnames[0]: 0, csv_fieldnames[1]: 0})


if __name__ == "__main__":
    # process_fletcher16(640)
    # process_jit_fletcher16_amortized_cost()
    # process_jit_fletcher16_execution_time()
    process_memory_access_checks()
    # process_program_sizes()
