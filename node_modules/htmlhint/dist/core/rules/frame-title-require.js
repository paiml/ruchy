"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.default = {
    id: 'frame-title-require',
    description: 'A <frame> or <iframe> element must have an accessible name.',
    init(parser, reporter) {
        parser.addListener('tagstart', (event) => {
            const tagName = event.tagName.toLowerCase();
            const mapAttrs = parser.getMapAttrs(event.attrs);
            const col = event.col + tagName.length + 1;
            if (tagName === 'frame' || tagName === 'iframe') {
                const role = mapAttrs['role'];
                if (role === 'presentation' || role === 'none') {
                    return;
                }
                const hasAriaLabel = 'aria-label' in mapAttrs && mapAttrs['aria-label'].trim() !== '';
                const hasAriaLabelledby = 'aria-labelledby' in mapAttrs &&
                    mapAttrs['aria-labelledby'].trim() !== '';
                const hasTitle = 'title' in mapAttrs && mapAttrs['title'].trim() !== '';
                if (!hasAriaLabel && !hasAriaLabelledby && !hasTitle) {
                    reporter.warn(`A <${tagName}> element must have an accessible name.`, event.line, col, this, event.raw);
                }
            }
        });
    },
};
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiZnJhbWUtdGl0bGUtcmVxdWlyZS5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uLy4uL3NyYy9jb3JlL3J1bGVzL2ZyYW1lLXRpdGxlLXJlcXVpcmUudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6Ijs7QUFFQSxrQkFBZTtJQUNiLEVBQUUsRUFBRSxxQkFBcUI7SUFDekIsV0FBVyxFQUFFLDZEQUE2RDtJQUMxRSxJQUFJLENBQUMsTUFBTSxFQUFFLFFBQVE7UUFDbkIsTUFBTSxDQUFDLFdBQVcsQ0FBQyxVQUFVLEVBQUUsQ0FBQyxLQUFLLEVBQUUsRUFBRTtZQUN2QyxNQUFNLE9BQU8sR0FBRyxLQUFLLENBQUMsT0FBTyxDQUFDLFdBQVcsRUFBRSxDQUFBO1lBQzNDLE1BQU0sUUFBUSxHQUFHLE1BQU0sQ0FBQyxXQUFXLENBQUMsS0FBSyxDQUFDLEtBQUssQ0FBQyxDQUFBO1lBQ2hELE1BQU0sR0FBRyxHQUFHLEtBQUssQ0FBQyxHQUFHLEdBQUcsT0FBTyxDQUFDLE1BQU0sR0FBRyxDQUFDLENBQUE7WUFFMUMsSUFBSSxPQUFPLEtBQUssT0FBTyxJQUFJLE9BQU8sS0FBSyxRQUFRLEVBQUUsQ0FBQztnQkFFaEQsTUFBTSxJQUFJLEdBQUcsUUFBUSxDQUFDLE1BQU0sQ0FBQyxDQUFBO2dCQUM3QixJQUFJLElBQUksS0FBSyxjQUFjLElBQUksSUFBSSxLQUFLLE1BQU0sRUFBRSxDQUFDO29CQUMvQyxPQUFNO2dCQUNSLENBQUM7Z0JBR0QsTUFBTSxZQUFZLEdBQ2hCLFlBQVksSUFBSSxRQUFRLElBQUksUUFBUSxDQUFDLFlBQVksQ0FBQyxDQUFDLElBQUksRUFBRSxLQUFLLEVBQUUsQ0FBQTtnQkFDbEUsTUFBTSxpQkFBaUIsR0FDckIsaUJBQWlCLElBQUksUUFBUTtvQkFDN0IsUUFBUSxDQUFDLGlCQUFpQixDQUFDLENBQUMsSUFBSSxFQUFFLEtBQUssRUFBRSxDQUFBO2dCQUMzQyxNQUFNLFFBQVEsR0FBRyxPQUFPLElBQUksUUFBUSxJQUFJLFFBQVEsQ0FBQyxPQUFPLENBQUMsQ0FBQyxJQUFJLEVBQUUsS0FBSyxFQUFFLENBQUE7Z0JBRXZFLElBQUksQ0FBQyxZQUFZLElBQUksQ0FBQyxpQkFBaUIsSUFBSSxDQUFDLFFBQVEsRUFBRSxDQUFDO29CQUNyRCxRQUFRLENBQUMsSUFBSSxDQUNYLE1BQU0sT0FBTyx5Q0FBeUMsRUFDdEQsS0FBSyxDQUFDLElBQUksRUFDVixHQUFHLEVBQ0gsSUFBSSxFQUNKLEtBQUssQ0FBQyxHQUFHLENBQ1YsQ0FBQTtnQkFDSCxDQUFDO1lBQ0gsQ0FBQztRQUNILENBQUMsQ0FBQyxDQUFBO0lBQ0osQ0FBQztDQUNNLENBQUEifQ==