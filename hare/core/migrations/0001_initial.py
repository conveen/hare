# Generated by Django 3.2.5 on 2021-08-15 15:19

import django.core.validators
from django.db import migrations, models
import django.db.models.deletion


class Migration(migrations.Migration):

    initial = True

    dependencies = [
    ]

    operations = [
        migrations.CreateModel(
            name='Destination',
            fields=[
                ('id', models.BigAutoField(auto_created=True, primary_key=True, serialize=False, verbose_name='ID')),
                ('url', models.CharField(max_length=2000, unique=True, validators=[django.core.validators.URLValidator()])),
                ('num_args', models.IntegerField()),
                ('is_fallback', models.BooleanField(db_index=True)),
                ('is_default_fallback', models.BooleanField(db_index=True, default=False)),
                ('description', models.TextField()),
            ],
            options={
                'db_table': 'destination',
            },
        ),
        migrations.CreateModel(
            name='HealthCheck',
            fields=[
                ('id', models.BigAutoField(auto_created=True, primary_key=True, serialize=False, verbose_name='ID')),
                ('check_field', models.BooleanField()),
            ],
            options={
                'db_table': 'database_health_check',
            },
        ),
        migrations.CreateModel(
            name='Alias',
            fields=[
                ('id', models.BigAutoField(auto_created=True, primary_key=True, serialize=False, verbose_name='ID')),
                ('name', models.CharField(max_length=100, unique=True)),
                ('destination', models.ForeignKey(on_delete=django.db.models.deletion.CASCADE, related_name='aliases', related_query_name='alias', to='core.destination')),
            ],
            options={
                'db_table': 'alias',
            },
        ),
    ]