// *************************************************************************************************
//  File: RogueScheduler.kt
//
//  A diminutive but fully-formed demonstration of idiomatic Kotlin.
//
//  1. Defines a sealed algebraic hierarchy to represent the discrete states of a task
//     transmogrifying through a rudimentary scheduler.
//  2. Employs a type-safe builder DSL to assemble a cohort of Task objects succinctly.
//  3. Utilises coroutines and the structured-concurrency discipline to execute tasks
//     concomitantly while preserving deterministic shutdown semantics.
// *************************************************************************************************

// ---------- Domain model -------------------------------------------------------------------------

/** Immutable value holder representing a unit of executable labor. */
data class Task(
    val id: Int,
    val description: String,
    val action: suspend () -> Unit,
)

/** Exhaustive taxonomy of execution outcomes; the `sealed` modifier ensures compiler‐enforced totality. */
sealed interface TaskResult {
    /** Successful completion carrying an optional payload. */
    data class Success(val id: Int, val elapsedMillis: Long) : TaskResult
    /** Recoverable misadventure accompanied by the causal `Throwable`. */
    data class Failure(val id: Int, val cause: Throwable) : TaskResult
    /** Voluntary cessation initiated by the caller before execution. */
    data class Cancelled(val id: Int) : TaskResult
}

// ---------- DSL for declarative task construction -----------------------------------------------

/** Fluent builder furnishing a terse, expressive syntax for batch task definition. */
@DslMarker
annotation class TaskDsl

@TaskDsl
class TaskBatchBuilder {
    private val tasks = mutableListOf<Task>()

    /** Registers a new task whose body is expressed as a suspending lambda. */
    fun task(description: String, block: suspend () -> Unit) {
        tasks += Task(tasks.size + 1, description, block)
    }

    internal fun build(): List<Task> = tasks.toList()
}

/** Conveniences the client with type inference and inline lambda to craft a task batch. */
fun taskBatch(init: TaskBatchBuilder.() -> Unit): List<Task> =
    TaskBatchBuilder().apply(init).build()

// ---------- Scheduler implementation -------------------------------------------------------------

import kotlinx.coroutines.*
import kotlin.system.*

/** Executes all tasks concurrently, returning a conglomerate of `TaskResult` artifacts. */
suspend fun runTasks(tasks: List<Task>): List<TaskResult> = coroutineScope {
    val startEpoch = System.currentTimeMillis()

    // Launch each task within its own child coroutine; Deferred encapsulates the eventual result.
    val futures: List<Deferred<TaskResult>> = tasks.map { task ->
        async {
            val elapsed = measureTimeMillis {
                try {
                    task.action()
                } catch (t: CancellationException) {
                    // Propagate structured-concurrency cancellation upward; annotate as `Cancelled`.
                    return@async TaskResult.Cancelled(task.id)
                } catch (t: Throwable) {
                    // Swallow domain-level exception, encapsulate in Failure result.
                    return@async TaskResult.Failure(task.id, t)
                }
            }
            // If the lambda returns normally, the endeavor is deemed triumphant.
            TaskResult.Success(task.id, elapsed)
        }
    }

    // Await completion of the entire cohort, preserving result order by task identifier.
    futures.awaitAll().sortedBy { result ->
        when (result) {
            is TaskResult.Success   -> result.id
            is TaskResult.Failure   -> result.id
            is TaskResult.Cancelled -> result.id
        }
    }
}

// ---------- Demonstration entry-point ------------------------------------------------------------

fun main() = runBlocking {
    // Compose an eclectic suite of tasks via the DSL.
    val tasks = taskBatch {
        task("Inconsequential delay") {
            delay(250)
            println("Task A executed on thread ${Thread.currentThread().name}")
        }
        task("Spurious exception") {
            delay(100)
            error("Intentional kaboom")
        }
        task("CPU-bound Fibonacci") {
            val n = 25
            val fib = generateSequence(0 to 1) { it.second to it.first + it.second }
                .take(n + 1).last().first
            println("fib($n) = $fib")
        }
    }

    println("Launching ${tasks.size} tasks concurrently…\n")

    // Drive the scheduler and acquire the final ledger.
    val ledger = runTasks(tasks)

    // Expository epilogue.
    println("\n────────── Execution Ledger ──────────")
    ledger.forEach { result ->
        when (result) {
            is TaskResult.Success ->
                println("✔︎ Task ${result.id} succeeded in ${result.elapsedMillis} ms")
            is TaskResult.Failure ->
                println("✘ Task ${result.id} failed with ${result.cause::class.simpleName}: ${result.cause.message}")
            is TaskResult.Cancelled ->
                println("⚑ Task ${result.id} was cancelled before commencement")
        }
    }
}
