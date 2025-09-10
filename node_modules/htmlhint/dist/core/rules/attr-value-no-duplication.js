"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = {
    id: 'attr-value-no-duplication',
    description: 'Class attributes should not contain duplicate values. Other attributes can be checked via configuration.',
    init(parser, reporter, options) {
        const defaultAttributesToCheck = ['class'];
        const attributesToCheck = Array.isArray(options)
            ? options
            : defaultAttributesToCheck;
        parser.addListener('tagstart', (event) => {
            const attrs = event.attrs;
            let attr;
            const col = event.col + event.tagName.length + 1;
            for (let i = 0, l = attrs.length; i < l; i++) {
                attr = attrs[i];
                const attrName = attr.name.toLowerCase();
                if (!attributesToCheck.includes(attrName)) {
                    continue;
                }
                if (!attr.value || !/\s/.test(attr.value)) {
                    continue;
                }
                const values = attr.value.trim().split(/\s+/);
                const duplicateMap = {};
                for (const value of values) {
                    if (value && duplicateMap[value] === true) {
                        reporter.error(`Duplicate value [ ${value} ] was found in attribute [ ${attr.name} ].`, event.line, col + attr.index, this, attr.raw);
                        break;
                    }
                    duplicateMap[value] = true;
                }
            }
        });
    },
};
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiYXR0ci12YWx1ZS1uby1kdXBsaWNhdGlvbi5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uLy4uL3NyYy9jb3JlL3J1bGVzL2F0dHItdmFsdWUtbm8tZHVwbGljYXRpb24udHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6Ijs7QUFFQSxrQkFBZTtJQUNiLEVBQUUsRUFBRSwyQkFBMkI7SUFDL0IsV0FBVyxFQUNULDBHQUEwRztJQUM1RyxJQUFJLENBQUMsTUFBTSxFQUFFLFFBQVEsRUFBRSxPQUFPO1FBRTVCLE1BQU0sd0JBQXdCLEdBQUcsQ0FBQyxPQUFPLENBQUMsQ0FBQTtRQUcxQyxNQUFNLGlCQUFpQixHQUFHLEtBQUssQ0FBQyxPQUFPLENBQUMsT0FBTyxDQUFDO1lBQzlDLENBQUMsQ0FBQyxPQUFPO1lBQ1QsQ0FBQyxDQUFDLHdCQUF3QixDQUFBO1FBRTVCLE1BQU0sQ0FBQyxXQUFXLENBQUMsVUFBVSxFQUFFLENBQUMsS0FBSyxFQUFFLEVBQUU7WUFDdkMsTUFBTSxLQUFLLEdBQUcsS0FBSyxDQUFDLEtBQUssQ0FBQTtZQUN6QixJQUFJLElBQUksQ0FBQTtZQUNSLE1BQU0sR0FBRyxHQUFHLEtBQUssQ0FBQyxHQUFHLEdBQUcsS0FBSyxDQUFDLE9BQU8sQ0FBQyxNQUFNLEdBQUcsQ0FBQyxDQUFBO1lBRWhELEtBQUssSUFBSSxDQUFDLEdBQUcsQ0FBQyxFQUFFLENBQUMsR0FBRyxLQUFLLENBQUMsTUFBTSxFQUFFLENBQUMsR0FBRyxDQUFDLEVBQUUsQ0FBQyxFQUFFLEVBQUUsQ0FBQztnQkFDN0MsSUFBSSxHQUFHLEtBQUssQ0FBQyxDQUFDLENBQUMsQ0FBQTtnQkFDZixNQUFNLFFBQVEsR0FBRyxJQUFJLENBQUMsSUFBSSxDQUFDLFdBQVcsRUFBRSxDQUFBO2dCQUd4QyxJQUFJLENBQUMsaUJBQWlCLENBQUMsUUFBUSxDQUFDLFFBQVEsQ0FBQyxFQUFFLENBQUM7b0JBQzFDLFNBQVE7Z0JBQ1YsQ0FBQztnQkFHRCxJQUFJLENBQUMsSUFBSSxDQUFDLEtBQUssSUFBSSxDQUFDLElBQUksQ0FBQyxJQUFJLENBQUMsSUFBSSxDQUFDLEtBQUssQ0FBQyxFQUFFLENBQUM7b0JBQzFDLFNBQVE7Z0JBQ1YsQ0FBQztnQkFHRCxNQUFNLE1BQU0sR0FBRyxJQUFJLENBQUMsS0FBSyxDQUFDLElBQUksRUFBRSxDQUFDLEtBQUssQ0FBQyxLQUFLLENBQUMsQ0FBQTtnQkFDN0MsTUFBTSxZQUFZLEdBQWlDLEVBQUUsQ0FBQTtnQkFFckQsS0FBSyxNQUFNLEtBQUssSUFBSSxNQUFNLEVBQUUsQ0FBQztvQkFDM0IsSUFBSSxLQUFLLElBQUksWUFBWSxDQUFDLEtBQUssQ0FBQyxLQUFLLElBQUksRUFBRSxDQUFDO3dCQUMxQyxRQUFRLENBQUMsS0FBSyxDQUNaLHFCQUFxQixLQUFLLCtCQUErQixJQUFJLENBQUMsSUFBSSxLQUFLLEVBQ3ZFLEtBQUssQ0FBQyxJQUFJLEVBQ1YsR0FBRyxHQUFHLElBQUksQ0FBQyxLQUFLLEVBQ2hCLElBQUksRUFDSixJQUFJLENBQUMsR0FBRyxDQUNULENBQUE7d0JBQ0QsTUFBSztvQkFDUCxDQUFDO29CQUNELFlBQVksQ0FBQyxLQUFLLENBQUMsR0FBRyxJQUFJLENBQUE7Z0JBQzVCLENBQUM7WUFDSCxDQUFDO1FBQ0gsQ0FBQyxDQUFDLENBQUE7SUFDSixDQUFDO0NBQ00sQ0FBQSJ9