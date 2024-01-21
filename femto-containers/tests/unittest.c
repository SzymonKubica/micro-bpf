#include <stdbool.h>
#include <stdio.h>
#include <stdint.h>
#include <assert.h>
#include <string.h>
#include <inttypes.h>
#include <sys/random.h>
#include <time.h>

#include "femtocontainer/femtocontainer.h"
#include "femtocontainer/instruction.h"

#ifndef NUM_RAND_TESTS
#define NUM_RAND_TESTS  10000
#endif

static uint8_t _f12r_stack[512];

/* 2 load, 1 under test and the return */
#define NUM_INSTRUCTIONS    4

bpf_instruction_t boilerplate[] = {
    {
        .opcode = 0x79, /* LDXDW */
        .dst = 0,
        .src = 1,
    },
    {
        .opcode = 0x79, /* LDXDW */
        .dst = 2,
        .src = 1,
        .offset = 8,
    }
};

typedef struct {
    int64_t arg1;
    int64_t arg2;
    int64_t result;
} test_context_t;

typedef struct {
    f12r_header_t header;
    uint8_t rodata[68];
    uint64_t text[NUM_INSTRUCTIONS + 1];
} test_application_t;

typedef struct {
    bpf_instruction_t instruction;
    test_context_t context;
    int64_t (*verify_func)(int64_t arg1, int64_t arg2);
    void (*prep_args)(int64_t *arg1, int64_t *arg2);
    bool no_imm_test;
} alu_test_content_t;

static int64_t _sum(int64_t arg1, int64_t arg2)
{
    return arg1 + arg2;
}

static int64_t _sub(int64_t arg1, int64_t arg2)
{
    return arg1 - arg2;
}

static int64_t _mul(int64_t arg1, int64_t arg2)
{
    return arg1 * arg2;
}

static int64_t _div(int64_t arg1, int64_t arg2)
{
    return (uint64_t)arg1 / (uint64_t)arg2;
}

static void _div_prep(int64_t *arg1, int64_t *arg2)
{
    if (*arg2 == 0) {
        *arg2 = 1;
    }
}

static int64_t _lsh(int64_t arg1, int64_t arg2)
{
    return (uint64_t)arg1 << (uint64_t)arg2;
}

static int64_t _rsh(int64_t arg1, int64_t arg2)
{
    return (uint64_t)arg1 >> (uint64_t)arg2;
}

static int64_t _arsh(int64_t arg1, int64_t arg2)
{
    return arg1 >> (uint64_t)arg2;
}

static void _shift_prep(int64_t *arg1, int64_t *arg2)
{
    *arg2 &= 0x1f;
}

static int64_t _or(int64_t arg1, int64_t arg2)
{
    return arg1 | arg2;
}

static int64_t _and(int64_t arg1, int64_t arg2)
{
    return (uint64_t)arg1 & (uint64_t)arg2;
}

static int64_t _neg(int64_t arg1, int64_t arg2)
{
    (void)arg2;
    return -arg1;
}

static int64_t _mod(int64_t arg1, int64_t arg2)
{
    return (uint64_t)arg1 % (uint64_t)arg2;
}

static int64_t _xor(int64_t arg1, int64_t arg2)
{
    return (uint64_t)arg1 ^ (uint64_t)arg2;
}

static int64_t _mov(int64_t arg1, int64_t arg2)
{
    return (uint64_t)arg2;
}

static const alu_test_content_t tests[] = {
    {
        .instruction = {
            .opcode = 0x0f,
            .src = 2,
        },
        .verify_func = _sum,
    },
    {
        .instruction = {
            .opcode = 0x1f,
            .src = 2,
        },
        .verify_func = _sub,
    },
    {
        .instruction = {
            .opcode = 0x2f,
            .src = 2,
        },
        .verify_func = _mul,
    },
    {
        .instruction = {
            .opcode = 0x3f,
            .src = 2,
        },
        .verify_func = _div,
        .prep_args = _div_prep,
    },
    {
        .instruction = {
            .opcode = 0x4f,
            .src = 2,
        },
        .verify_func = _or,
    },
    {
        .instruction = {
            .opcode = 0x5f,
            .src = 2,
        },
        .verify_func = _and,
    },
    {
        .instruction = {
            .opcode = 0x6f,
            .src = 2,
        },
        .verify_func = _lsh,
        .prep_args = _shift_prep,
    },
    {
        .instruction = {
            .opcode = 0x7f,
            .src = 2,
        },
        .verify_func = _rsh,
        .prep_args = _shift_prep,
    },
    {
        .instruction = {
            .opcode = 0x8f,
        },
        .verify_func = _neg,
        .no_imm_test = true,
    },
    {
        .instruction = {
            .opcode = 0x9f,
            .src = 2,
        },
        .verify_func = _mod,
    },
    {
        .instruction = {
            .opcode = 0xaf,
            .src = 2,
        },
        .verify_func = _xor,
    },
    {
        .instruction = {
            .opcode = 0xbf,
            .src = 2,
        },
        .verify_func = _mov,
    },
    {
        .instruction = {
            .opcode = 0xcf,
            .src = 2,
        },
        .verify_func = _arsh,
        .prep_args = _shift_prep,
    },
};


#define NUM_TESTS   (sizeof(tests)/sizeof(alu_test_content_t))

static test_application_t test_app;

static void add_instruction(const bpf_instruction_t *instr, test_application_t *test_app)
{
    test_app->header.data_len = 0;
    test_app->header.rodata_len = 68;
    test_app->header.text_len = sizeof(uint64_t) * NUM_INSTRUCTIONS;

    memcpy(&test_app->text[0], boilerplate, sizeof(boilerplate));
    memcpy(&test_app->text[2], instr, sizeof(bpf_instruction_t));
    static const bpf_instruction_t return_instr = {
        .opcode = BPF_INSTRUCTION_CLS_BRANCH | BPF_INSTRUCTION_BRANCH_EXIT
    };
    memcpy(&test_app->text[NUM_INSTRUCTIONS - 1], &return_instr, sizeof(bpf_instruction_t));
}

int main()
{
    time_t curtime = time(0);
    printf("Rand seed: %lu\n", curtime);
    srand(curtime);
    for (size_t idx = 0; idx < NUM_TESTS; idx++) {
        const alu_test_content_t *test = &tests[idx];
        add_instruction(&test->instruction, &test_app);

        for (size_t i = 0; i < NUM_RAND_TESTS; i++) {
            test_context_t ctx;
            getrandom(&ctx.arg1, sizeof(ctx.arg1), 0);
            getrandom(&ctx.arg2, sizeof(ctx.arg1), 0);

            if (test->prep_args) {
                test->prep_args(&ctx.arg1, &ctx.arg2);
            }

            f12r_t femtoc = {
                .application = (uint8_t*)&test_app,
                .application_len = sizeof(test_app),
                .stack = _f12r_stack,
                .stack_size = sizeof(_f12r_stack),
            };

            f12r_setup(&femtoc);
            int64_t res = 0;
            int result = f12r_execute_ctx(&femtoc,
                                          (void*)&ctx,
                                          sizeof(test_context_t), &res);
            int64_t expected = test->verify_func(ctx.arg1, ctx.arg2);
            if (result != FC_OK || res != expected) {
                printf("idx: %lu, Opcode: 0x%x, %"PRIi64", %"PRIi64" = %"PRIi64", got: %"PRIi64"\n",
                        idx,
                        test->instruction.opcode,
                        ctx.arg1,
                        ctx.arg2,
                        res,
                        expected);
            }

            assert(result == FC_OK);
            assert(res == expected);
        }
        if (test->no_imm_test) {
            continue;
        }
        bpf_instruction_t instruction = test->instruction;

        for (size_t i = 0; i < NUM_RAND_TESTS; i++) {
            instruction.opcode = instruction.opcode & ~0x08; /* clear source bit */
            test_context_t ctx;
            getrandom(&ctx.arg1, sizeof(ctx.arg1), 0);
            getrandom(&ctx.arg2, sizeof(ctx.arg2), 0);

            if (test->prep_args) {
                test->prep_args(&ctx.arg1, &ctx.arg2);
            }

            ctx.arg2 = (int32_t)(uint32_t)(UINT32_MAX & ctx.arg2); /* Truncate to 32 bit */
            instruction.immediate = (int32_t)(uint32_t)ctx.arg2;

            add_instruction(&instruction, &test_app);

            f12r_t femtoc = {
                .application = (uint8_t*)&test_app,
                .application_len = sizeof(test_app),
                .stack = _f12r_stack,
                .stack_size = sizeof(_f12r_stack),
            };

            f12r_setup(&femtoc);
            int64_t res = 0;
            int result = f12r_execute_ctx(&femtoc,
                                          (void*)&ctx,
                                          sizeof(test_context_t), &res);
            int64_t expected = test->verify_func(ctx.arg1, ctx.arg2);
            if (result != FC_OK || res != expected) {
                printf("idx: %lu, Opcode: 0x%x, %"PRIi64", %"PRIi64" = %"PRIi64", got: %"PRIi64"\n",
                        idx,
                        instruction.opcode,
                        ctx.arg1,
                        ctx.arg2,
                        res,
                        expected);
            }

            assert(result == FC_OK);
            assert(res == expected);
        }
    }

    return 0;
}
