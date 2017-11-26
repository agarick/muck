/*
  proj: scream
  by:   agaric
  copy:
  - cc by-nc-sa 3.0 (https://creativecommons.org/licenses/by-nc-sa/3.0/)
  - this file "main.c" is an adaptation of the source code provided at
    http://buildyourownlisp.com/, written by Daniel Holden.
  desc:
  - this project is an ongoing pre-pre alpha stage of building a scheme,
    going through tutorials, experimenting with open code, etc.
 */

#include <stdio.h>
#include <stdlib.h>
#include <editline/readline.h>
#include <histedit.h>
#include "mpc.h"

enum { LVAL_NUM, LVAL_ERR };
enum { LERR_DIV_ZERO, LERR_BAD_OP, LERR_BAD_NUM };

typedef struct {
  int type;
  long num;
  int err;
} lval;

lval lval_num(long x) {
  lval v;
  v.type = LVAL_NUM;
  v.num = x;
  return v;
}

lval lval_err(int x) {
  lval v;
  v.type = LVAL_ERR;
  v.err = x;
  return v;
}

void lval_print(lval v) {
  switch (v.type) {
  case LVAL_NUM: printf("%li", v.num); break;
  case LVAL_ERR:
    if (v.err == LERR_DIV_ZERO) {
      printf("error: division by zero");
    }
    if (v.err == LERR_BAD_OP) {
      printf("error: invalid operator");
    }
    if (v.err == LERR_BAD_NUM) {
      printf("error: invalid number");
    }
    break;
  }
}

void lval_println(lval v) { lval_print(v); putchar('\n'); }

int number_of_nodes(mpc_ast_t* t) {
  if (t->children_num == 0) { return 1; }
  if (t->children_num >= 1) {
    int total = 1;
    for (int i = 0; i < t->children_num; i++) {
      total = total + number_of_nodes(t->children[i]);
    }
    return total;
  }
  return 0;
}

int number_of_branches(mpc_ast_t* t) {
  if (t->children_num == 0) { return 0; }
  if (t->children_num >= 1) {
    int total = 1;
    for (int i = 0; i < t->children_num; i++) {
      total = total + number_of_branches(t->children[i]);
    }
    return total;
  }
  return 0;
}

int most_nodes(mpc_ast_t* t) {
  if (t->children_num == 0) { return 0; }
  if (t->children_num >= 1) {
    int max = t->children_num;
    for (int i = 0; i < t->children_num; i++) {
      int next = most_nodes(t->children[i]);
      if (next > max) { max = next; }
    }
    return max;
  }
  return 0;
}

lval eval_op(lval x, char* op, lval y) {
  if (x.type == LVAL_ERR) { return x; }
  if (y.type == LVAL_ERR) { return y; }

  if (strcmp(op, "+") == 0) { return lval_num(x.num + y.num); }
  if (strcmp(op, "-") == 0) { return lval_num(x.num - y.num); }
  if (strcmp(op, "*") == 0) { return lval_num(x.num * y.num); }
  if (strcmp(op, "%") == 0) { return lval_num(x.num % y.num); }
  if (strcmp(op, "/") == 0) {
    return y.num == 0
      ? lval_err(LERR_DIV_ZERO)
      : lval_num(x.num / y.num);
  }
  if (strcmp(op, "^") == 0) {
    int total = 1;
    for (int i = 0; i < y.num; i++) {
      total = total * x.num;
    }
    return lval_num(total);
  }
  return lval_err(LERR_BAD_OP);
}

lval eval(mpc_ast_t* t) {
  if (strstr(t->tag, "number")) {
    errno = 0;
    long x = strtol(t->contents, NULL, 10);
    return errno != ERANGE ? lval_num(x) : lval_err(LERR_BAD_NUM);
  }
  char* op = t->children[1]->contents;
  lval x = eval(t->children[2]);
  int i = 3;
  while (strstr(t->children[i]->tag, "expr")) {
    x = eval_op(x, op, eval(t->children[i]));
    i++;
  }
  return x;
}

int main(int argc, char** argv) {

  // create parsers
  mpc_parser_t* Number     = mpc_new("number");
  mpc_parser_t* Operator   = mpc_new("operator");
  mpc_parser_t* Expression = mpc_new("expression");
  mpc_parser_t* Program    = mpc_new("program");

  // define parsers with language
  mpca_lang(MPCA_LANG_DEFAULT,
            "                                                            \
              number     : /-?[0-9]+/ ;                                  \
              operator   : '+' | '-' | '*' | '/' | '%' | '^' ;           \
              expression : <number> | '(' <operator> <expression>+ ')' ; \
              program    : /^/ <operator> <expression>+ /$/ ;            \
            ",
            Number, Operator, Expression, Program);

  puts("scream - C-c to exit\n");

  while (1) {
    char* input = readline("scream> ");
    add_history(input);

    mpc_result_t r;
    if (mpc_parse("<stdin>", input, Program, &r)) {
      lval result = eval(r.output);
      lval_println(result);
      mpc_ast_delete(r.output);
    } else {
      mpc_err_print(r.error);
      mpc_err_delete(r.error);
    }

    free(input);
  }

  // undefine and delete parsers
  mpc_cleanup(4, Number, Operator, Expression, Program);

  return 0;
}
