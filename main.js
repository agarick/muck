// WIP

/*
   proj: scream
   by:   agaric
   copy:
   - https://github.com/maryrosecook/littlelisp/blob/master/LICENSE
   - This file ("main.js") is an adaptation of the source code provided at
     https://maryrosecook.com/blog/post/little-lisp-interpreter, written by
     Mary Rose Cook.
 */

function tokenise(input) {
  return input.replace(/\(/g, ' ( ')
              .replace(/\)/g, ' ) ')
              .trim()
              .split(/\s+/);
}

function categorise(input) {
  if (!isNaN(parseFloat(input))) {
    return { type: 'lit', val: parseFloat(input) };
  } else {
    return { type: 'id', val: input };
  }
}

function parenthesise(input, list) {
  if (list === undefined) {
    return parenthesise(input, []);
  } else {
    var token = input.shift();
    if (token === undefined) {
      return list.pop();
    } else if (token === '(') {
      list.push(parenthesise(input, []));
      return parenthesise(input, list);
    } else if (token === ')') {
      return list;
    } else {
      return parenthesise(input, list.concat(categorise(token)));
    }
  }
}

var Context = function(scope, parent) {
  this.scope = scope;
  this.parent = parent;
  this.get = function(id) {
    if (id in this.scope) {
      return this.scope[id];
    } else if (this.parent !== undefined) {
      return this.parent.get(id);
    }
  };
};

function parse(input) {
  return parenthesise(tokenise(input));
}

var library = {
  first: function(x) {
    return x[0];
  },
  rest: function(x) {
    return x.slice(1);
  },
  print: function(x) {
    console.log(x);
    return x;
  }
};

var special = {
  lambda: function(input, context) {
    return function() {
      var lambdaArguments = arguments;
      var lambdaScope = input[1].reduce(function(acc, x, i) {
        acc[x.value] = lambdaArguments[i];
        return acc;
      }, {});
      return interpret(input[2], new Context(lambdaScope, context));
    };
  }
};

function interpret(input, context) {
  if (context === undefined) {
    return interpret(input, new Context(library));
  } else if (input instanceof Array) {
    return interpretList(input, context);
  } else if (input.type === 'id') {
    return interpretId(input, context);
  } else {
    return input.val;
  }
}

function interpretList(input, context) {
  if (input.length > 0 && input[0].val in special) {
    return special[input[0].val](input, context);
  } else {
    var list = input.map(function(x) {
      return interpret(x, context);
    });
    if (list[0].type === 'function') {
      return list[0].val(list.slice(1));
    } else {
      return list;
    }
  }
}

var createInvocation = function(input, context) {
  return {
    type: 'function',
    val: function(args) {
      return context.get(input.val).apply(null, args);
    }
  };
};

function interpretId(input, context) {
  return context.get(input.val) instanceof Function ?
         createInvocation(input, context) :
         context.get(input.val);
}

var readline = require('readline');
var rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false
});

console.log('welcome to scream');
rl.on('line', function(line) {
  console.log(interpret(parse(line)));
});
