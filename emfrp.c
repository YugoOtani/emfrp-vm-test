#include <stdio.h>
#include <stdlib.h>
#define MAX_NODE_SIZE 128
#define DEBUG
typedef unsigned char uint8_t;
typedef union
{
    void *ptr;
    int num;
} value_t;
typedef void (*dev_input_t)(int *);
enum bytecode
{
    BC_None = 0,
    BC_Nil = 1,
    BC_Int = 2,
    BC_Bool = 3,
    BC_Add = 4,
    BC_Mul = 5,
    BC_Je8 = 6,
    BC_Je32 = 7,
    BC_J8 = 8,
    BC_J32 = 9,
    BC_GetLocal = 10,
    BC_SetLocal = 11,
    BC_AllocNode = 12,
    BC_AllocNodeNew = 13,
    BC_UpdateNode = 14,
    BC_GetNode = 15,
    BC_SetNode = 16,
    BC_GetLast = 17,
    BC_SaveLast = 18,
    BC_AllocFunc = 19,
    BC_AllocFuncNew = 20,
    BC_AllocData = 21,
    BC_AllocDataNew = 22,
    BC_Return = 23,
    BC_Call = 24,
    BC_Exit = 25,
    BC_Halt = 26,

};

typedef enum exec_result_t
{
    OK,
    RUNTIME_ERR,
    PANIC,
    TODO,
} exec_result_t;
typedef struct input_action_t
{
    enum
    {
        ACTION_NONE,
        INSN,
        DEV
    } kind;
    union
    {
        uint8_t *insns;
        dev_input_t dev;
    };

} input_action_t;

typedef void (*output_action_t)(int *);
typedef output_action_t dev_output_t;
typedef struct node_t
{
    int vlast;
    int v;
    output_action_t o_action;
    input_action_t i_action;
    struct node_t *next;
} node_t;
void set_input_action(int node_index, dev_input_t driver);
void set_output_action(int node_index, dev_output_t driver);
exec_result_t emfrp_exec(uint8_t *p);
void emfrp_set_new_code(uint8_t *p);

static uint8_t *update;
static value_t stack[128];
static node_t *nodes_head, *nodes_tail;
int next_int(uint8_t **p)
{ // little endian
    int ret = (int)(**p) + (((int)(p[0][1])) << 8) + (((int)(p[0][2])) << 16) + (((int)(p[0][3])) << 24);
    *p += 4;
    return ret;
}
uint8_t next_byte(uint8_t **p)
{
    uint8_t ret = **p;
    *p += 1;
    return ret;
}
node_t *node_b(uint8_t n)
{
    node_t *p = nodes_head;
    for (uint8_t i = 0; i < n; i++)
    {
        p = p->next;
    }
    return p;
}

void set_input_action(int node_index, dev_input_t driver)
{
}
void set_output_action(int node_index, dev_output_t driver)
{
}
void print_node(char *s)
{
    printf("%s\n", s);
    for (node_t *p = nodes_head; p != NULL; p = p->next)
    {
        printf(" v:%d vlast:%d kind:%d \n", p->v, p->vlast, p->i_action.kind);
    }
}
exec_result_t emfrp_exec(uint8_t *p)
{
    value_t *rbp = &stack[0];
    value_t *rsp = &stack[0];
    value_t *tmp_v;
    node_t *tmp_nd;
    uint8_t tmp_byte;
    uint8_t *tmp_byte_p;
    int tmp_int;

    while (1)
    {
#ifdef DEBUG
        printf("%d ", *p);
#endif
        switch (*p)
        {
        case BC_None:
            return PANIC;
        case BC_Nil:
            rsp->ptr = 0;
            ++rsp;
            ++p;
            break;
        case BC_Int:
            ++p;
            rsp->num = next_int(&p);
            ++rsp;
            break;
        case BC_Bool:
            ++p;
            rsp->num = next_byte(&p);
            break;
        case BC_Add: // a b rsp -> (a+b) rsp
            --rsp;
            (rsp - 1)->num += rsp->num;
            ++p;
            break;
        case BC_Mul:
            --rsp;
            (rsp - 1)->num *= rsp->num;
            ++p;
            break;
        case BC_J8:
            ++p;
            tmp_byte = next_byte(&p);
            p += tmp_byte;
            break;
        case BC_Je8:
            --rsp;

            if (rsp->num)
            {
                ++p;
                tmp_byte = next_byte(&p);
                p += tmp_byte;
            }
            else
            {
                p += 2;
            }
            break;
        case BC_AllocNode: // ALLOCNODE offset insnlen insns
            ++p;
            --rsp;
            tmp_byte = next_byte(&p); // node offset
            tmp_int = next_int(&p);   // insnlen
            tmp_nd = node_b(tmp_byte);
            free(tmp_nd->i_action.insns);
            tmp_nd->v = rsp->num; // 前回の値を引き継ぐかどうか
            tmp_byte_p = (uint8_t *)malloc(tmp_int);
            for (int i = 0; i < tmp_int; ++i)
            {
                tmp_byte_p[i] = *p;
                ++p;
            }
            tmp_nd->i_action.insns = tmp_byte_p;
            break;
        case BC_AllocNodeNew:
            ++p;
            --rsp;
            tmp_int = next_int(&p); //
            tmp_nd = (node_t *)malloc(sizeof(node_t));
            tmp_nd->i_action.insns = (uint8_t *)malloc(sizeof(tmp_int));
            tmp_nd->i_action.kind = INSN;
            tmp_nd->o_action = NULL;
            tmp_nd->next = NULL;
            tmp_nd->v = rsp->num;
            tmp_byte_p = (uint8_t *)malloc(tmp_int);
            for (int i = 0; i < tmp_int; ++i)
            {
                tmp_byte_p[i] = *p;
                ++p;
            }
            tmp_nd->i_action.insns = tmp_byte_p;
            if (nodes_tail == NULL)
            {
                nodes_head = nodes_tail = tmp_nd;
            }
            else
            {
                nodes_tail->next = tmp_nd;
            }
            break;
        case BC_GetLocal:
            ++p;
            *rsp = *((value_t *)rbp->ptr + next_byte(&p));
            ++rsp;
            break;

        case BC_SetLocal:
            --rsp;
            ++p;
            *((value_t *)rbp->ptr + next_byte(&p)) = *rsp;
            break;
        case BC_UpdateNode:
            ++p;
            tmp_byte = next_byte(&p);

            tmp_nd = node_b(tmp_byte);
            switch (tmp_nd->i_action.kind)
            {
            case DEV:
                tmp_nd->i_action.dev(&tmp_nd->v);
                break;
            case ACTION_NONE:
                break;
            case INSN:
                rsp->ptr = (void *)rbp;
                (rsp + 1)->ptr = (void *)p;
                rsp += 2;
                p = tmp_nd->i_action.insns;
                break;
            }
            break;
        case BC_SetNode:
            --rsp;
            ++p;
            tmp_byte = next_byte(&p);
            node_b(tmp_byte)->v = rsp->num;
            break;
        case BC_GetNode:
            ++p;
            tmp_byte = next_byte(&p);
            rsp->num = node_b(tmp_byte)->v;
            ++rsp;
            break;
        case BC_GetLast:
            ++p;
            tmp_byte = next_byte(&p);
            rsp->num = node_b(tmp_byte)->vlast;
            ++rsp;
            break;
        case BC_SaveLast:
            for (node_t *nd_p = nodes_head; nd_p != NULL; nd_p = nd_p->next)
            {
                nd_p->vlast = nd_p->v;
            }
            ++p;
            break;
        case BC_Return: // rbp rip ret_val rsp
            rsp -= 2;
            rbp->ptr = (value_t *)(rsp - 1)->ptr;
            p = (uint8_t *)rsp->ptr;
            *(rsp - 1) = *(rsp + 1);

            break;
        case BC_Halt:
            if (rsp == &stack[0])
                return OK;
            else
                return PANIC;
        case BC_Exit:
            if (&stack[1] == rsp)
            {
                return OK;
            }
            else
            {
                return PANIC;
            }
        default:
            return TODO;
        }
#ifdef DEBUG
        if ((rsp - &stack[0]) < 0 || 128 <= (rsp - &stack[0]))
        {
            return PANIC;
        }
#endif
    }
}

void emfrp_set_new_code(uint8_t *code)
{
    int init_len = next_int(&code);
    int upd_len = next_int(&code);

    if (init_len != 0)
    {
        emfrp_exec(code);
#ifdef DEBUG
        printf("\n\n");
#endif
    }
    if (upd_len != 0)
    {
        code += init_len;
        if (update != NULL)
            free(update);
        uint8_t *upd = (uint8_t *)malloc(upd_len);
        for (int i = 0; i < upd_len; ++i)
        {
            upd[i] = code[i];
        }
        update = upd;
    }
}
int main(void)
{
    uint8_t code[] = {28, 0, 0, 0, 6, 0, 0, 0, 2, 0, 0, 0, 0, 13, 17, 0, 0, 0, 15, 0, 6, 7, 2, 1, 0, 0, 0, 8, 5, 2, 0, 0, 0, 0, 23, 26, 18, 14, 0, 16, 0, 26};
    emfrp_set_new_code(code);
    for (int i = 0; i < 10; i++)
    {
        if (emfrp_exec(update) == OK)
        {
#ifdef DEBUG
            printf("\n\n");
            print_node("node info");
#endif
            continue;
        }
        else
        {
            printf("ABORT\n");
            return 1;
        }
    }
}
