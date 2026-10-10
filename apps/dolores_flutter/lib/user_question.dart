import 'package:flutter/material.dart';

import 'chat.dart';
import 'theme.dart';

class UserQuestionCard extends StatefulWidget {
  final ChatController chat;
  const UserQuestionCard({super.key, required this.chat});
  @override
  State<UserQuestionCard> createState() => _UserQuestionCardState();
}

class _UserQuestionCardState extends State<UserQuestionCard> {
  final custom = TextEditingController();
  int index = -1;
  String? callId;
  void sync() {
    final chat = widget.chat;
    if (index == chat.questionIndex && callId == chat.userQuestion?['callId']) {
      return;
    }
    index = chat.questionIndex;
    callId = chat.userQuestion?['callId'] as String?;
    final options =
        (chat.userQuestion!['questions'] as List)[index]['options'] as List;
    final answer = chat.questionAnswers[index];
    custom.text = options.any((o) => o['label'] == answer) ? '' : answer;
  }

  @override
  void dispose() {
    custom.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final chat = widget.chat;
    sync();
    final questions = chat.userQuestion!['questions'] as List;
    final question = questions[index] as Map;
    final options = question['options'] as List;
    final locked = chat.answeringQuestion || chat.stopping;
    final answer = chat.questionAnswers[index];
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return Container(
      key: const Key('user-question'),
      width: double.infinity,
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: p.soft,
        border: Border.all(color: p.border),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Expanded(
                child: Text(
                  'Answer question',
                  style: TextStyle(fontWeight: FontWeight.w600),
                ),
              ),
              if (questions.length > 1)
                Text(
                  '${index + 1} of ${questions.length}',
                  style: TextStyle(color: p.muted),
                ),
            ],
          ),
          const SizedBox(height: 8),
          ConstrainedBox(
            constraints: BoxConstraints(
              maxHeight: (MediaQuery.sizeOf(context).height * .38).clamp(
                120.0,
                300.0,
              ),
            ),
            child: SingleChildScrollView(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  SelectableText('${question['question']}'),
                  const SizedBox(height: 10),
                  for (var i = 0; i < options.length; i++)
                    Padding(
                      padding: const EdgeInsets.only(bottom: 6),
                      child: OutlinedButton(
                        key: Key('question-option-$i'),
                        style: OutlinedButton.styleFrom(
                          alignment: Alignment.centerLeft,
                          padding: const EdgeInsets.all(12),
                          backgroundColor:
                              answer == options[i]['label'] &&
                                  custom.text.isEmpty
                              ? p.surface
                              : null,
                        ),
                        onPressed: locked
                            ? null
                            : () {
                                custom.clear();
                                chat.setQuestionAnswer(
                                  index,
                                  '${options[i]['label']}',
                                );
                              },
                        child: Row(
                          children: [
                            Icon(
                              answer == options[i]['label'] &&
                                      custom.text.isEmpty
                                  ? Icons.radio_button_checked
                                  : Icons.radio_button_unchecked,
                              size: 18,
                            ),
                            const SizedBox(width: 10),
                            Expanded(
                              child: Column(
                                crossAxisAlignment: CrossAxisAlignment.start,
                                children: [
                                  Text('${options[i]['label']}'),
                                  if ('${options[i]['description'] ?? ''}'
                                      .isNotEmpty)
                                    Text(
                                      '${options[i]['description']}',
                                      style: TextStyle(
                                        color: p.muted,
                                        fontSize: 12,
                                      ),
                                    ),
                                ],
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),
                  TextField(
                    key: const Key('question-custom-answer'),
                    controller: custom,
                    enabled: !locked,
                    minLines: 1,
                    maxLines: 3,
                    decoration: const InputDecoration(
                      hintText: 'Or type your own answer',
                      border: OutlineInputBorder(),
                    ),
                    onChanged: (value) => chat.setQuestionAnswer(index, value),
                  ),
                  if (chat.questionError != null)
                    Padding(
                      padding: const EdgeInsets.only(top: 8),
                      child: Text(
                        chat.questionError!,
                        style: TextStyle(
                          color: Theme.of(context).colorScheme.error,
                        ),
                      ),
                    ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 10),
          Wrap(
            spacing: 8,
            runSpacing: 4,
            children: [
              if (index > 0)
                TextButton(
                  onPressed: locked ? null : () => chat.showQuestion(index - 1),
                  child: const Text('Back'),
                ),
              FilledButton(
                key: const Key('submit-question-answer'),
                onPressed: locked || answer.trim().isEmpty
                    ? null
                    : index < questions.length - 1
                    ? () => chat.showQuestion(index + 1)
                    : chat.questionAnswers.any((a) => a.trim().isEmpty)
                    ? null
                    : chat.answerQuestion,
                child: Text(
                  chat.answeringQuestion
                      ? 'Sending…'
                      : index < questions.length - 1
                      ? 'Next'
                      : 'Send answer',
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
