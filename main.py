# WIP

# proj: scream
# by:   agaric
# copy:
# - https://github.com/norvig/pytudes/blob/master/LICENSE
# - This file ("main.py") is an adaptation of the source code provided at
#   https://github.com/norvig/pytudes/blob/master/py/lis.py, written by
#   Peter Norvig.

import math
import operator as op

Symbol = str
List = list
Number = (int, float)

def parse(program):
    "Read an expression from a string."
    return read_from_tokens(tokenise(program))

def tokenise(s):
    "Convert a string into a list of tokens."
    return s.replace('(', ' ( ').replace(')', ' ) ').split()

def read_from_tokens(tokens):
    "Read an expression from a sequence of tokens."
    if len(tokens) == 0:
        raise SyntaxError('unexpected EOF')
    token = tokens.pop(0)
    if '(' == token:
        L = []
        while ')' != tokens[0]:
            L.append(read_from_tokens(tokens))
        tokens.pop(0)
        return L
    elif ')' == token:
        raise SyntaxError('unexpected )')
    else:
        return atom(token)

def atom(token):
    "Numbers become numbers; every other token is a symbol."
    try:
        return int(token)
    except ValueError:
        try:
            return float(token)
        except ValueError:
            return Symbol(token)

def standard_env():
    "An environment with some standard procedures."
    env = Env()
    env.update(vars(math))
    env.update({
        '+': op.add, '-': op.sub, '*': op.mul, '/': op.truediv,
        '>': op.gt, '<': op.lt, '>=': op.ge, '<=': op.le, '=': op.eq,
        'abs': abs,
        'append': op.add,
        'apply': lambda proc, args: proc(*args),
        'begin': lambda *x: x[-1],
        'car': lambda x: x[0],
        'cdr': lambda x: x[1:],
        'cons': lambda x,y: [x] + y,
        'eq?': op.is_,
        'expt': pow,
        'equal?': op.eq,
        'length': len,
        'list': lambda *x: List(x),
        'list?': lambda x: isinstance(x, List),
        'map': map,
        'max': max,
        'min': min,
        'not': op.not_,
        'null?': lambda x: x == [],
        'number?': lambda x: isinstance(x, Number),
        'print': print,
        'procedure?': callable,
        'round': round,
        'symbol?': lambda x: isinstance(x, Symbol),
    })
    return env

class Env(dict):
    "An environment: a dict of {'var': val} pairs, with an outer Env."
    def __init__(self, params=(), args=(), outer=None):
        self.update(zip(params, args))
        self.outer = outer
    def find(self, var):
        "Find the innermost Env where var appears."
        return self if (var in self) else self.outer.find(var)

global_env = standard_env()

def repl(prompt='scream> '):
    "A prompt-read-eval-print loop."
    while True:
        val = eval(parse(input(prompt)))
        if val is not None:
            print(lispstr(val))

def lispstr(exp):
    "Convert a Python object back into Lisp-readable string."
    if isinstance(exp, List):
        return '(' + ' '.join(map(lispstr, exp)) + ')'
    else:
        return str(exp)

class Procedure(object):
    "A user-defined procedure."
    def __init__(self, params, body, env):
        self.params, self.body, self.env = params, body, env
    def __call__(self, *args):
        return eval(self.body, Env(self.params, args, self.env))

def eval(x, env=global_env):
    "Evaluate an expression in an environment."
    if isinstance(x, Symbol):
        return env.find(x)[x]
    elif not isinstance(x, List):
        return x
    elif x[0] == 'quote':
        (_, exp) = x
        return exp
    elif x[0] == 'if':
        (_, test, conseq, alt) = x
        exp = (conseq if eval(test, env) else alt)
        return eval(exp, env)
    elif x[0] == 'define':
        (_, var, exp) = x
        env.find(var)[var] = eval(exp, env)
    elif x[0] == 'lambda':
        (_, params, body) = x
        return Procedure(params, body, env)
    else:
        proc = eval(x[0], env)
        args = [eval(arg, env) for arg in x[1:]]
        return proc(*args)

repl()
