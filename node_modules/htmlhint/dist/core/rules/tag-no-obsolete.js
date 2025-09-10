"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const OBSOLETE_TAGS = [
    'applet',
    'acronym',
    'bgsound',
    'dir',
    'frame',
    'frameset',
    'noframes',
    'isindex',
    'keygen',
    'listing',
    'menuitem',
    'nextid',
    'noembed',
    'plaintext',
    'rb',
    'rtc',
    'strike',
    'xmp',
    'basefont',
    'big',
    'blink',
    'center',
    'font',
    'marquee',
    'multicol',
    'nobr',
    'spacer',
    'tt',
];
exports.default = {
    id: 'tag-no-obsolete',
    description: 'Disallows the use of obsolete HTML tags.',
    init(parser, reporter, _options) {
        parser.addListener('tagstart,tagend', (event) => {
            const tagName = event.tagName.toLowerCase();
            if (OBSOLETE_TAGS.includes(tagName)) {
                reporter.error(`The tag [ ${event.tagName} ] is obsolete in HTML5 and should not be used.`, event.line, event.col, this, event.raw);
            }
        });
    },
};
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoidGFnLW5vLW9ic29sZXRlLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vLi4vc3JjL2NvcmUvcnVsZXMvdGFnLW5vLW9ic29sZXRlLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiI7O0FBR0EsTUFBTSxhQUFhLEdBQUc7SUFDcEIsUUFBUTtJQUNSLFNBQVM7SUFDVCxTQUFTO0lBQ1QsS0FBSztJQUNMLE9BQU87SUFDUCxVQUFVO0lBQ1YsVUFBVTtJQUNWLFNBQVM7SUFDVCxRQUFRO0lBQ1IsU0FBUztJQUNULFVBQVU7SUFDVixRQUFRO0lBQ1IsU0FBUztJQUNULFdBQVc7SUFDWCxJQUFJO0lBQ0osS0FBSztJQUNMLFFBQVE7SUFDUixLQUFLO0lBQ0wsVUFBVTtJQUNWLEtBQUs7SUFDTCxPQUFPO0lBQ1AsUUFBUTtJQUNSLE1BQU07SUFDTixTQUFTO0lBQ1QsVUFBVTtJQUNWLE1BQU07SUFDTixRQUFRO0lBQ1IsSUFBSTtDQUNMLENBQUE7QUFFRCxrQkFBZTtJQUNiLEVBQUUsRUFBRSxpQkFBaUI7SUFDckIsV0FBVyxFQUFFLDBDQUEwQztJQUN2RCxJQUFJLENBQUMsTUFBTSxFQUFFLFFBQVEsRUFBRSxRQUFRO1FBQzdCLE1BQU0sQ0FBQyxXQUFXLENBQUMsaUJBQWlCLEVBQUUsQ0FBQyxLQUFLLEVBQUUsRUFBRTtZQUM5QyxNQUFNLE9BQU8sR0FBRyxLQUFLLENBQUMsT0FBTyxDQUFDLFdBQVcsRUFBRSxDQUFBO1lBRTNDLElBQUksYUFBYSxDQUFDLFFBQVEsQ0FBQyxPQUFPLENBQUMsRUFBRSxDQUFDO2dCQUNwQyxRQUFRLENBQUMsS0FBSyxDQUNaLGFBQWEsS0FBSyxDQUFDLE9BQU8saURBQWlELEVBQzNFLEtBQUssQ0FBQyxJQUFJLEVBQ1YsS0FBSyxDQUFDLEdBQUcsRUFDVCxJQUFJLEVBQ0osS0FBSyxDQUFDLEdBQUcsQ0FDVixDQUFBO1lBQ0gsQ0FBQztRQUNILENBQUMsQ0FBQyxDQUFBO0lBQ0osQ0FBQztDQUNNLENBQUEifQ==