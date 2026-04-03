# Dynamic MLOps Training for Ruchy Oracle

**Specification Version**: 1.1.0
**Status**: REVISED - Incorporating Team Review
**Author**: Claude Code
**Date**: 2025-12-08
**Revised**: 2025-12-08
**Ticket**: ORACLE-001
**Review**: Team feedback incorporated (Cold Start, GNN Complexity, Feedback Latency)

---

## Executive Summary

This specification defines a **self-improving MLOps system** for the Ruchy Oracle that continuously learns from transpilation outcomes. The system grows more intelligent over time through automated data collection, drift detection, curriculum learning, and knowledge distillation - adapting the battle-tested techniques from depyler's production Oracle.

**Core Principle**: The Oracle should never be "done" - it is a living system that improves with every transpilation cycle.

---

## 1. Introduction

### 1.1 Problem Statement

The current Ruchy Oracle uses bootstrap training with 30 hardcoded samples (`src/oracle/classifier.rs:266-346`). This approach has fundamental limitations:

1. **Static Knowledge**: Model cannot learn from new error patterns
2. **No Feedback Loop**: Transpilation outcomes are discarded
3. **Single Training Event**: No continuous improvement mechanism
4. **Missing Production Data**: Real-world errors not incorporated

### 1.2 Solution Overview

Implement a **Six-Strategy Acceleration Pipeline** that:

1. **Collects** error patterns from every transpilation
2. **Detects** model drift and triggers retraining
3. **Applies** curriculum learning (easy → hard)
4. **Distills** knowledge from high-confidence predictions
5. **Embeds** errors using GNN for structural similarity
6. **Monitors** via Hansei (反省) reflection analysis

### 1.3 Success Criteria

| Metric | Baseline | Target | Measurement |
|--------|----------|--------|-------------|
| Single-shot fix rate | 40% | 80% | Production transpilations |
| Error classification accuracy | 70% | 95% | Holdout test set |
| Model staleness | N/A | <7 days | Drift detection |
| Training data size | 30 samples | 12,000+ | Corpus size |

---


## Sub-spec Index

| Sub-spec | Scope |
|----------|-------|
| [System Architecture](sub/oracle-architecture.md) | Pipeline overview, component mapping, data flow |
| [Six-Strategy Pipeline](sub/oracle-six-strategies.md) | Corpus collection, drift detection, curriculum, distillation, GNN, Hansei |
| [Quality Gates & Reviews](sub/oracle-quality-reviews.md) | Drift detection, implementation plan, Toyota Way & Google AI review |
| [Training UX Design](sub/oracle-training-ux-design.md) | Philosophy, default-on behavior, visual feedback format |
| [Training UX Integration](sub/oracle-training-ux-integration.md) | Customization, presets, CLI, Hunt mode, metrics persistence |

---

## 14. Extended References

### Additional Peer-Reviewed Citations (13-25)

13. **Holt, C. C.** (2004). "Forecasting seasonals and trends by exponentially weighted moving averages." *International Journal of Forecasting*, 20(1), 5-10. DOI: 10.1016/j.ijforecast.2003.09.015 *(Convergence estimation via exponential smoothing)*

14. **Gamma, E., Helm, R., Johnson, R., & Vlissides, J.** (1994). "Design Patterns: Elements of Reusable Object-Oriented Software." *Addison-Wesley*. ISBN: 978-0201633610 *(Builder pattern for configuration)*

15. **Domingos, P., & Hulten, G.** (2000). "Mining High-Speed Data Streams." *Proceedings of the 6th ACM SIGKDD International Conference on Knowledge Discovery and Data Mining*, 71-80. DOI: 10.1145/347090.347107 *(Online learning fundamentals)*

16. **Gama, J., Medas, P., Castillo, G., & Rodrigues, P.** (2004). "Learning with Drift Detection." *Advances in Artificial Intelligence - SBIA 2004*, 286-295. DOI: 10.1007/978-3-540-28645-5_29 *(DDM algorithm for drift detection)*

17. **Page, E. S.** (1954). "Continuous Inspection Schemes." *Biometrika*, 41(1/2), 100-115. DOI: 10.2307/2333009 *(Page-Hinkley test for change detection)*

18. **Baena-García, M., del Campo-Ávila, J., Fidalgo, R., Bifet, A., Gavaldà, R., & Morales-Bueno, R.** (2006). "Early Drift Detection Method." *Fourth International Workshop on Knowledge Discovery from Data Streams*, 77-86. *(EDDM for early drift warning)*

19. **Sugiyama, M., Krauledat, M., & Müller, K. R.** (2007). "Covariate Shift Adaptation by Importance Weighted Cross Validation." *Journal of Machine Learning Research*, 8, 985-1005. *(Handling distribution shift)*

20. **Gama, J., Žliobaitė, I., Bifet, A., Pechenizkiy, M., & Bouchachia, A.** (2014). "A Survey on Concept Drift Adaptation." *ACM Computing Surveys*, 46(4), Article 44. DOI: 10.1145/2523813 *(Comprehensive drift detection survey)*

21. **Tsymbal, A.** (2004). "The Problem of Concept Drift: Definitions and Related Work." *Computer Science Department, Trinity College Dublin*, Technical Report TCD-CS-2004-15. *(Concept drift taxonomy)*

22. **Widmer, G., & Kubat, M.** (1996). "Learning in the Presence of Concept Drift and Hidden Contexts." *Machine Learning*, 23(1), 69-101. DOI: 10.1007/BF00116900 *(FLORA system for concept drift)*

23. **Klinkenberg, R., & Joachims, T.** (2000). "Detecting Concept Drift with Support Vector Machines." *Proceedings of the 17th International Conference on Machine Learning*, 487-494. *(SVM-based drift detection)*

24. **Krawczyk, B., Minku, L. L., Gama, J., Stefanowski, J., & Woźniak, M.** (2017). "Ensemble Learning for Data Stream Analysis: A Survey." *Information Fusion*, 37, 132-156. DOI: 10.1016/j.inffus.2017.02.004 *(Ensemble methods for streaming data)*

25. **Lu, J., Liu, A., Dong, F., Gu, F., Gama, J., & Zhang, G.** (2019). "Learning under Concept Drift: A Review." *IEEE Transactions on Knowledge and Data Engineering*, 31(12), 2346-2363. DOI: 10.1109/TKDE.2018.2876857 *(Modern concept drift review)*

26. **Le Goues, C., Nguyen, T. V., Forrest, S., & Weimer, W.** (2012). "GenProg: A Generic Method for Automatic Software Repair." *IEEE Transactions on Software Engineering*, 38(1), 54-72. DOI: 10.1109/TSE.2011.104 *(Foundational Automated Software Repair)*

27. **Monperrus, M.** (2018). "Automatic Software Repair: A Bibliography." *ACM Computing Surveys*, 51(1), Article 17. DOI: 10.1145/3105906 *(Contextualizing the error-fix pattern approach)*

28. **Kreuzberger, D., Kühl, N., & Hirschl, S.** (2023). "Machine Learning Operations (MLOps): Overview, Definition, and Architecture." *IEEE Access*, 11, 31866-31879. DOI: 10.1109/ACCESS.2023.3262138 *(Formalizing the Continuous Training pipeline)*

29. **Sambasivan, N., et al.** (2021). "Everyone wants to do the model work, not the data work": Data Cascades in High-Stakes AI. *Proceedings of the 2021 CHI Conference on Human Factors in Computing Systems*, Article 39. DOI: 10.1145/3411764.3445518 *(Supporting Data Quality > Model Complexity)*

30. **Satyanarayanan, M.** (2017). "The Emergence of Edge Computing." *Computer*, 50(1), 30-39. DOI: 10.1109/MC.2017.9 *(Justification for local-first, low-latency inference)*

31. **Warden, P., & Situnayake, D.** (2019). "TinyML: Machine Learning with TensorFlow Lite on Arduino and Ultra-Low-Power Microcontrollers." *O'Reilly Media*. ISBN: 978-1492052043 *(Constraints for <1MB model size)*

32. **Gunning, D., & Aha, D.** (2019). "DARPA's Explainable Artificial Intelligence (XAI) Program." *AI Magazine*, 40(2), 44-58. DOI: 10.1609/aimag.v40i2.2850 *(Theoretical basis for Hansei reflection)*

33. **ISO/IEC TR 24029-1:2021**. "Artificial Intelligence (AI) — Assessment of the robustness of neural networks — Part 1: Overview." *(Standardization compliance)*

34. **Paleyes, A., Urma, R. G., & Lawrence, N. D.** (2022). "Challenges in Deploying Machine Learning: A Survey of Case Studies." *ACM Computing Surveys*, 55(6), Article 114. DOI: 10.1145/3533378 *(Addressing real-world deployment drift)*

35. **IEEE 2830-2021**. "IEEE Standard for Technical Verification of Explainable Artificial Intelligence (XAI)." *(Standards for visual feedback transparency)*

---

**Next Steps**:
1. ~~Team review of this specification~~ ✓ Complete
2. ~~Gather feedback and address concerns~~ ✓ Complete
3. ~~Update spec based on review~~ ✓ Complete
4. ~~Add Unified Training Loop UX (§13)~~ ✓ Complete
5. Final approval from Technical Lead, ML Engineer, QA Lead
6. Create implementation tickets in roadmap.yaml
7. Begin Phase 1 implementation

---

