import chalk from 'chalk';
import promptSync from 'prompt-sync';
import { execSync } from 'child_process';

const prompt = promptSync();

export function Shell() {
  console.log(chalk.green('Welcome to PartermShell! Type "exit" to quit.\n'));

  while (true) {
    // Display the customized prompt
    const input = prompt(chalk.blue('[partermai ~]$ '));

    // Exit the shell
    if (input.trim() === 'exit') {
      console.log(chalk.red('Exiting NixShell...'));
      break;
    }

    // Handle empty input
    if (!input.trim()) {
      continue;
    }

    try {
      // Execute the input as a shell command
      const output = execSync(input, { stdio: 'pipe' }).toString();
      console.log(chalk.green(output));
    } catch (err) {
      // Handle errors and display message
      console.log(chalk.red(`Error executing command: ${err.message}`));
    }
  }
}
