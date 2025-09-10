import { ArtifactLocation, Region, Result } from 'sarif';

import { SarifResultOptions } from '../types/node-sarif-builder';

import { setOptionValues } from './utils';

export class SarifResultBuilder {
  // Default result value
  result: Result = {
    level: 'error',
    message: {},
    ruleId:
      'SARIF_BUILDER_INVALID: Please send the rule Id ruleId property, or call setRuleId(ruleId)',
  };

  // Initialize SARIF Result builder
  constructor(options: SarifResultOptions = {}) {
    setOptionValues(options, this.result);
  }

  initSimple(options: {
    level: Result.level;
    messageText: string;
    ruleId: string;
    fileUri?: string;
    startLine?: number;
    startColumn?: number;
    endLine?: number;
    endColumn?: number;
  }) {
    this.setLevel(options.level);
    this.setMessageText(options.messageText);
    this.setRuleId(options.ruleId);
    if (options.fileUri) {
      this.setLocationArtifactUri({ uri: options.fileUri });
    }
    if (options.startLine !== null && options.startLine !== undefined) {
      // Initialize Region with default values with necessary
      const region: Region = {
        startLine: options.startLine,
        startColumn: options.startColumn || 1,
        endLine: options.endLine || options.startLine,
        endColumn: options.endColumn || 1,
      };
      // Check for invalid region values
      if (
        options.startLine === 0 ||
        options.startColumn === 0 ||
        options.endLine === 0 ||
        options.endColumn === 0
      ) {
        throw new Error(
          'Region limit can not be 0 (minimum line 1 or column 1) in ' +
            JSON.stringify(options),
        );
      }
      this.setLocationRegion(region);
    }
    return this;
  }

  setLevel(level: Result.level) {
    this.result.level = level;
  }

  setMessageText(message: string) {
    this.result.message.text = message;
  }

  setRuleId(ruleId: string) {
    this.result.ruleId = ruleId;
  }

  setLocationRegion(region: Region) {
    this.manageInitPhysicalLocation();
    this.result.locations[0].physicalLocation.region = region;
  }

  setLocationArtifactUri(artifactLocation: ArtifactLocation) {
    this.manageInitPhysicalLocation();
    this.result.locations[0].physicalLocation.artifactLocation =
      artifactLocation;
  }

  private manageInitLocation() {
    if (this.result?.locations?.length) {
      return;
    }
    this.result.locations = [{}];
  }

  private manageInitPhysicalLocation() {
    this.manageInitLocation();
    if (this.result?.locations[0].physicalLocation) {
      return;
    }
    this.result.locations[0].physicalLocation = {};
  }
}
