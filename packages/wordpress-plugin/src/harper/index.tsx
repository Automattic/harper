import { PluginSidebar, PluginSidebarMoreMenuItem } from '@wordpress/edit-post';
import { registerPlugin } from '@wordpress/plugins';
import Logo from './Logo';
import SidebarControl from './SidebarControl';
import './index.css';
import LinterProvider from './LinterProvider';

function Sidebar() {
	return (
		<>
			<PluginSidebarMoreMenuItem target="harper-sidebar" icon={Logo()}>
				Harper
			</PluginSidebarMoreMenuItem>
			<PluginSidebar name="harper-sidebar" title="Harper" icon={Logo}>
				<LinterProvider>
					<SidebarControl />
				</LinterProvider>
			</PluginSidebar>
		</>
	);
}

// @ts-expect-error
if (!window.__harperSidebarRegistered) {
	registerPlugin('harper-sidebar', { render: Sidebar });
	// @ts-expect-error
	window.__harperSidebarRegistered = true;
}
