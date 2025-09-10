import * as path from 'path';

import * as fs from 'fs-extra';
import { Log, Run } from 'sarif';

import { LogOptions } from '../types/node-sarif-builder';

import { EXTENSIONS_LANGUAGES } from './languages';
import { SarifRunBuilder } from './sarif-run-builder';
import { setOptionValues } from './utils';

// SARIF Builder
export class SarifBuilder {
  // Default run value
  log: Log = {
    $schema: 'http://json.schemastore.org/sarif-2.1.0.json',
    version: '2.1.0',
    runs: [],
  };

  // Initialize SARIF Log builder
  constructor(options: LogOptions = {}) {
    setOptionValues(options, this.log);
  }

  addRun(sarifRunBuilder: SarifRunBuilder) {
    this.log.runs.push(sarifRunBuilder.run);
  }

  generateSarifFileSync(file: string) {
    const sarifJsonString = this.buildSarifJsonString();
    fs.writeFileSync(file, sarifJsonString, 'utf8');
  }

  async generateSarifFile(file: string) {
    const sarifJsonString = this.buildSarifJsonString();
    await fs.writeFile(file, sarifJsonString, 'utf8');
  }

  buildSarifOutput() {
    // Complete runs
    this.log.runs = this.log.runs.map((run) => this.completeRunFields(run));
    return this.log;
  }

  // Build final sarif json, complete when possible
  buildSarifJsonString(options = { indent: false }) {
    this.buildSarifOutput();
    const sarifJson = options.indent
      ? JSON.stringify(this.log, null, 2)
      : JSON.stringify(this.log);
    if (sarifJson.includes('SARIF_BUILDER_INVALID')) {
      throw new Error(
        'Your SARIF log is invalid, please solve SARIF_BUILDER_INVALID messages',
      );
    }
    return sarifJson;
  }

  completeRunFields(run: Run): Run {
    // Collect all missing artifacts from results
    run.artifacts = run.artifacts || [];
    for (const result of run.results) {
      for (const location of result.locations || []) {
        if (
          location?.physicalLocation?.artifactLocation?.uri &&
          run.artifacts.filter(
            (artifact) =>
              artifact?.location?.uri ===
              location.physicalLocation.artifactLocation.uri,
          ).length === 0
        ) {
          // Add result to driver artifact only if not existing
          const ext = path
            .extname(location.physicalLocation.artifactLocation.uri)
            .replace('.', '');
          const language = EXTENSIONS_LANGUAGES[ext] || 'unknown';
          run.artifacts.push({
            sourceLanguage: language,
            location: { uri: location.physicalLocation.artifactLocation.uri },
          });
        }
      }
    }
    // Build artifacts indexes
    const artifactIndexes = run.artifacts.map((artifact) => {
      return artifact?.location?.uri;
    });

    // Build rules indexes
    const rulesIndexes = (run?.tool?.driver?.rules || []).map((rule) => {
      return rule.id;
    });

    // Update index in results with computed values
    run.results = run.results.map((result) => {
      // Set rule index in results
      if (rulesIndexes.indexOf(result.ruleId) > -1) {
        result.ruleIndex = rulesIndexes.indexOf(result.ruleId);
      }
      // Set artifact index in results
      if (result.locations) {
        result.locations = result.locations.map((location) => {
          const uri = location?.physicalLocation?.artifactLocation?.uri;
          if (uri && artifactIndexes.indexOf(uri) > -1) {
            location.physicalLocation.artifactLocation.index =
              artifactIndexes.indexOf(uri);
          }
          return location;
        });
      }
      return result;
    });

    return run;
  }
}
