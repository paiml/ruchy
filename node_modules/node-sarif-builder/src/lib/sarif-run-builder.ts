import { Run } from 'sarif';

import { SarifRuleBuilder } from '..';
import { SarifRunOptions } from '../types/node-sarif-builder';

import { SarifResultBuilder } from './sarif-result-builder';
import { setOptionValues } from './utils';

// SARIF Run builder
export class SarifRunBuilder {
  // Default run value
  run: Run = {
    tool: {
      driver: {
        name:
          process.env.npm_package_name ||
          'SARIF_BUILDER_INVALID: Please send the tool name in tool.driver.name property, or call setToolName(name)',
        rules: [],
      },
    },
    results: [],
  };

  // Initialize SARIF Run builder
  constructor(options: SarifRunOptions = {}) {
    setOptionValues(options, this.run);
  }

  initSimple(options: {
    toolDriverName: string;
    toolDriverVersion: string;
    url?: string;
  }) {
    this.setToolDriverName(options.toolDriverName);
    this.setToolDriverVersion(options.toolDriverVersion);
    if (options.url) {
      this.setToolDriverUri(options.url);
    }
    return this;
  }

  addRule(sarifRuleBuilder: SarifRuleBuilder) {
    this.run.tool.driver.rules.push(sarifRuleBuilder.rule);
  }

  addResult(sarifResultBuilder: SarifResultBuilder) {
    this.run.results.push(sarifResultBuilder.result);
  }

  setToolDriverName(name: string) {
    this.run.tool.driver.name = name;
  }

  setToolDriverVersion(version: string) {
    this.run.tool.driver.version = version;
  }

  setToolDriverUri(url: string) {
    this.run.tool.driver.informationUri = url;
  }
}
