export default function generateGreeting(): string {
	const timeOfDay = getTimeOfDay();

	switch (timeOfDay) {
		case TimeOfDay.Morning:
			return 'Good morning!';
		case TimeOfDay.Afternoon:
			return 'Good afternoon!';
		case TimeOfDay.Evening:
			return 'Good evening!';
	}
}

enum TimeOfDay {
	Morning = 0,
	Afternoon = 1,
	Evening = 2,
}

function getTimeOfDay(date: Date = new Date()): TimeOfDay {
	const hour = date.getHours();

	if (hour >= 5 && hour < 12) return TimeOfDay.Morning;
	if (hour >= 12 && hour < 18) return TimeOfDay.Afternoon;
	return TimeOfDay.Evening;
}
