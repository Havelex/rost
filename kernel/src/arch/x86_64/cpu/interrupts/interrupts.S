.altmacro
.section .text

# --- Helper Macro ---
.macro generate_isr num
    .global isr\num
    isr\num:
        # Error codes are pushed by CPU for: 8, 10-14, 17, 21
        .if \num == 8 || (\num >= 10 && \num <= 14) || \num == 17 || \num == 21
            # CPU already pushed error_code, now we push the vector
            pushq $\num
        .else
            # No error code pushed by CPU, so we push a dummy 0 first, then vector
            pushq $0
            pushq $\num
        .endif
        jmp common_isr
.endm

# --- Wrapper to force evaluation ---
.macro isr_wrapper i
    generate_isr %i
.endm

# --- Generate Exceptions (0-31) ---
.set i, 0
.rept 32
    isr_wrapper %i
    .set i, i + 1
.endr

# --- Generate Hardware Interrupts (32-47) ---
.set i, 32
.rept 16
    isr_wrapper %i
    .set i, i + 1
.endr

# --- Common Handler ---
.extern x86_64_interrupt_handler
common_isr:
    pushq %rax
    pushq %rbx
    pushq %rcx
    pushq %rdx
    pushq %rsi
    pushq %rdi
    pushq %rbp
    pushq %r8
    pushq %r9
    pushq %r10
    pushq %r11
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15

    # 1. Capture pointer to r15
    movq %rsp, %rdi

    # 2. Align to 16-bytes (15 regs + 2 macro = 17. 17*8 = 136. 136 is not div by 16)
    # Subtraction is required here for ABI compliance.
    subq $8, %rsp

    call x86_64_interrupt_handler

    # 3. Undo alignment
    addq $8, %rsp

    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %r11
    popq %r10
    popq %r9
    popq %r8
    popq %rbp
    popq %rdi
    popq %rsi
    popq %rdx
    popq %rcx
    popq %rbx
    popq %rax

    # 4. Cleanup vector/error code
    addq $16, %rsp
    iretq

# --- Stub Table ---
.section .data
.align 8
.global isr_stub_table

.macro table_entry i
    .quad isr\i
.endm

isr_stub_table:
    .set i, 0
    .rept 48
        table_entry %i
        .set i, i + 1
    .endr
