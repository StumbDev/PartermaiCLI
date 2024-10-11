var shell = require('shelljs');
const { Command } = require('commander');
const { colors } = require ('yoctocolors');
const { promptSync } = require('prompt-sync');
const { Shell } = require('./shellMode.mjs');
const { partermEdit } = require('./editorMode.mjs');
const prompt = promptSync();
const program = new Command();

console.log(colors.bold('Partermai Version 1 Dev ⚡'))

program
  .name('Partemai CLI ⚡')
  .version('0.0.1 codename flake')
  .description('The multi-functional CLI')
program.command('+editor')
  .description('Open the cli editor')
  .action(partermEdit())
program.command('+version')
    .description('Show version')
    .action(function() {
         console.log(colors.bold(''))
    })
program.command('+shell')
    .description('start shell mode')
    .action(Shell())
program.parse(args, command);
