import { Linter, WorkerLinter } from 'harper.js';
import React, {
	createContext,
	ReactNode,
	useContext,
	useEffect,
	useRef,
	useState,
} from 'react';

const linterContext = createContext<Linter>(new WorkerLinter());

export default function LinterProvider({
	children,
}: {
	children: ReactNode | ReactNode[];
}) {
	const linter = useRef(new WorkerLinter());

	return (
		<linterContext.Provider value={linter.current}>
			{children}
		</linterContext.Provider>
	);
}

export function useLinter(): Linter {
	return useContext(linterContext);
}

export function useLintDescriptions(): Record<string, string> {
	const linter = useLinter();
	const [descriptions, setDescriptions] = useState({});

	useEffect(() => {
		linter.getLintDescriptions().then(setDescriptions);
	}, [linter]);

	return descriptions;
}
