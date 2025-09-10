import { ReportingDescriptor } from 'sarif';

import { SarifRuleOptions } from '../types/node-sarif-builder';

import { setOptionValues } from './utils';

/*
  Rules describing any error that the linter can return
*/
export class SarifRuleBuilder {
  rule: ReportingDescriptor = {
    id: 'SARIF_BUILDER_INVALID: Please send the rule identifier in id property, or call setRuleId(ruleId)',
    shortDescription: {
      text: 'SARIF_BUILDER_INVALID: Please send the rule text in shortDescription.text property, or call setShortDescriptionText(text)',
    },
  };

  // Initialize SARIF Run builder
  constructor(options: SarifRuleOptions = {}) {
    setOptionValues(options, this.rule);
  }

  initSimple(options: {
    ruleId: string;
    shortDescriptionText: string;
    fullDescriptionText?: string;
    helpUri?: string;
  }) {
    this.setRuleId(options.ruleId);
    if (options.shortDescriptionText) {
      this.setShortDescriptionText(options.shortDescriptionText);
    }
    if (options.fullDescriptionText) {
      this.setFullDescriptionText(options.fullDescriptionText);
    }
    if (options.helpUri) {
      this.setHelpUri(options.helpUri);
    }
    return this;
  }

  setRuleId(ruleId: string) {
    this.rule.id = ruleId;
  }

  setShortDescriptionText(shortDescriptionText: string) {
    this.rule.shortDescription.text = shortDescriptionText;
  }

  setFullDescriptionText(fullDescriptionText: string) {
    this.rule.fullDescription = this.rule.fullDescription || { text: '' };
    this.rule.fullDescription.text = fullDescriptionText;
  }

  setHelpUri(url: string) {
    this.rule.helpUri = url;
  }
}
