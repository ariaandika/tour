# Tour

HTML Templating in rust

## Problems

- macro based template yield the best performance and memory
- but recompilation in develpment is annoying
- runtime based template does not have recompilation time
- but it bring overhead in engines and memory

## What Tour template brings

parts of the best of each world, implemented to the best case

- runtime template bring the smallest overhead by having simple rules, resulting no compilation required
- compiled template provide rich expressions with native performance via macros

## Going in depth

runtime template can only use layout and render variables

runtime template should be used in content heavy page with less logic like layouts

compiled template can do pretty much anything rust can do, like pattern matching

compiled template should be used in logic heavy page like list and tables

in practice, single page is a runtime template with fields of compiled template

