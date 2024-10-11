import chalk from 'chalk';
import promptSync from 'prompt-sync';
import fs from 'fs';

const prompt = promptSync({ sigint: true });

export function partermEdit() {
  console.log(chalk.green('Welcome to PartermEdit!'));
  console.log(chalk.yellow('Start typing your code. Press CTRL + D when finished.\n'));

  let code = '';
  let line;

  // Read input until CTRL + D (EOF)
  try {
    while (true) {
      line = prompt(''); // Empty prompt for continuous input
      code += line + '\n';
    }
  } catch (err) {
    // CTRL + D sends EOF, triggering an exception we use to finish input
    console.log(chalk.blue('\nInput finished.'));

    // After CTRL + D, prompt for a filename
    const filename = prompt(chalk.green('Enter filename to save: '));
    if (filename) {
      fs.writeFileSync(filename, code);
      console.log(chalk.green(`Code saved to ${filename}`));
    } else {
      console.log(chalk.red('No filename provided. Code not saved.'));
    }
  }
}
