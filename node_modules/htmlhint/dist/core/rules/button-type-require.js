"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = {
    id: 'button-type-require',
    description: 'The type attribute of a <button> element must be present with a valid value: "button", "submit", or "reset".',
    init(parser, reporter) {
        parser.addListener('tagstart', (event) => {
            const tagName = event.tagName.toLowerCase();
            if (tagName === 'button') {
                const mapAttrs = parser.getMapAttrs(event.attrs);
                const col = event.col + tagName.length + 1;
                if (mapAttrs.type === undefined) {
                    reporter.warn('The type attribute must be present on <button> elements.', event.line, col, this, event.raw);
                }
                else {
                    const typeValue = mapAttrs.type.toLowerCase();
                    if (typeValue !== 'button' &&
                        typeValue !== 'submit' &&
                        typeValue !== 'reset') {
                        reporter.warn('The type attribute of <button> must have a valid value: "button", "submit", or "reset".', event.line, col, this, event.raw);
                    }
                }
            }
        });
    },
};
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiYnV0dG9uLXR5cGUtcmVxdWlyZS5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uLy4uL3NyYy9jb3JlL3J1bGVzL2J1dHRvbi10eXBlLXJlcXVpcmUudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6Ijs7QUFHQSxrQkFBZTtJQUNiLEVBQUUsRUFBRSxxQkFBcUI7SUFDekIsV0FBVyxFQUNULDhHQUE4RztJQUNoSCxJQUFJLENBQUMsTUFBa0IsRUFBRSxRQUFrQjtRQUN6QyxNQUFNLENBQUMsV0FBVyxDQUFDLFVBQVUsRUFBRSxDQUFDLEtBQVksRUFBUSxFQUFFO1lBQ3BELE1BQU0sT0FBTyxHQUFHLEtBQUssQ0FBQyxPQUFPLENBQUMsV0FBVyxFQUFFLENBQUE7WUFFM0MsSUFBSSxPQUFPLEtBQUssUUFBUSxFQUFFLENBQUM7Z0JBQ3pCLE1BQU0sUUFBUSxHQUFHLE1BQU0sQ0FBQyxXQUFXLENBQUMsS0FBSyxDQUFDLEtBQUssQ0FBQyxDQUFBO2dCQUNoRCxNQUFNLEdBQUcsR0FBRyxLQUFLLENBQUMsR0FBRyxHQUFHLE9BQU8sQ0FBQyxNQUFNLEdBQUcsQ0FBQyxDQUFBO2dCQUUxQyxJQUFJLFFBQVEsQ0FBQyxJQUFJLEtBQUssU0FBUyxFQUFFLENBQUM7b0JBQ2hDLFFBQVEsQ0FBQyxJQUFJLENBQ1gsMERBQTBELEVBQzFELEtBQUssQ0FBQyxJQUFJLEVBQ1YsR0FBRyxFQUNILElBQUksRUFDSixLQUFLLENBQUMsR0FBRyxDQUNWLENBQUE7Z0JBQ0gsQ0FBQztxQkFBTSxDQUFDO29CQUNOLE1BQU0sU0FBUyxHQUFHLFFBQVEsQ0FBQyxJQUFJLENBQUMsV0FBVyxFQUFFLENBQUE7b0JBQzdDLElBQ0UsU0FBUyxLQUFLLFFBQVE7d0JBQ3RCLFNBQVMsS0FBSyxRQUFRO3dCQUN0QixTQUFTLEtBQUssT0FBTyxFQUNyQixDQUFDO3dCQUNELFFBQVEsQ0FBQyxJQUFJLENBQ1gseUZBQXlGLEVBQ3pGLEtBQUssQ0FBQyxJQUFJLEVBQ1YsR0FBRyxFQUNILElBQUksRUFDSixLQUFLLENBQUMsR0FBRyxDQUNWLENBQUE7b0JBQ0gsQ0FBQztnQkFDSCxDQUFDO1lBQ0gsQ0FBQztRQUNILENBQUMsQ0FBQyxDQUFBO0lBQ0osQ0FBQztDQUNGLENBQUEifQ==