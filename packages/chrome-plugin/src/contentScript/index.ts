import '@webcomponents/custom-elements';
import $ from 'jquery';
import TextareaHighlight from '../TextareaHighlight';

console.info('contentScript is running');

$('textarea').each(function () {
	new TextareaHighlight(this);
});
